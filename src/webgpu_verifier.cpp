/*
 * Copyright (C) 2023-2025 Ligero, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <algorithm>
#include <cctype>
#include <charconv>
#include <chrono>
#include <cstring>
#include <cstdlib>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <format>
#include <iostream>
#include <memory>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string_view>
#include <unordered_set>

#if !defined(_WIN32)
#include <fcntl.h>
#include <unistd.h>
#endif

#include <transpiler.hpp>
#include <invoke.hpp>
#include <runtime.hpp>
#include <wgpu.hpp>

#include <util/portable_sample.hpp>
#include <util/boost/portable_binary_iarchive.hpp>
#include <zkp/common.hpp>
#include <zkp/finite_field_gmp.hpp>
#include <zkp/nonbatch_context.hpp>
#include <interpreter.hpp>

#include <boost/algorithm/hex.hpp>
#include <boost/iostreams/filtering_stream.hpp>
#include <boost/iostreams/filter/gzip.hpp>
#include <wabt/error-formatter.h>
#include <wabt/wast-parser.h>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

using namespace wabt;
using namespace ligero;
using namespace ligero::vm;
namespace io = boost::iostreams;
namespace fs = std::filesystem;

using field_t = zkp::bn254_gmp;
using executor_t = webgpu_context;
using buffer_t = typename executor_t::buffer_type;

constexpr bool enable_RAM = false;

namespace {
struct DaemonError final : public std::runtime_error {
    using std::runtime_error::runtime_error;
};

[[noreturn]] void fail(bool daemon_mode, const std::string& msg) {
    if (daemon_mode) {
        throw DaemonError(msg);
    }
    std::cerr << msg << std::endl;
    std::exit(EXIT_FAILURE);
}

static std::vector<u8> decode_base64_arg_or_fail(std::string_view in, bool daemon_mode) {
    // RFC 4648 base64 decoder (lenient: strips ASCII whitespace; supports '+'/'/' and URL-safe '-'/'_').
    // Returns raw bytes with no terminator.

    // 1) strip whitespace and reject embedded NULs (JSON strings should not contain them, but be safe).
    std::string s;
    s.reserve(in.size());
    for (unsigned char c : in) {
        if (c == '\0') {
            fail(daemon_mode, "Error: bytes_b64 arg contains embedded NUL byte");
        }
        if (std::isspace(c) != 0) {
            continue;
        }
        s.push_back(static_cast<char>(c));
    }

    if (s.empty()) {
        return {};
    }

    const size_t rem = s.size() % 4;
    if (rem == 1) {
        fail(daemon_mode, "Error: invalid base64 length (mod 4 == 1)");
    }
    if (rem != 0) {
        // Accept unpadded base64 by adding '=' padding.
        s.append(4 - rem, '=');
    }

    auto val = [](unsigned char c) -> int {
        if (c >= 'A' && c <= 'Z') return static_cast<int>(c - 'A');
        if (c >= 'a' && c <= 'z') return static_cast<int>(c - 'a' + 26);
        if (c >= '0' && c <= '9') return static_cast<int>(c - '0' + 52);
        if (c == '+' || c == '-') return 62;  // '-' = URL-safe '+'
        if (c == '/' || c == '_') return 63;  // '_' = URL-safe '/'
        return -1;
    };

    std::vector<u8> out;
    out.reserve((s.size() / 4) * 3);

    for (size_t i = 0; i < s.size(); i += 4) {
        const unsigned char c0 = static_cast<unsigned char>(s[i]);
        const unsigned char c1 = static_cast<unsigned char>(s[i + 1]);
        const unsigned char c2 = static_cast<unsigned char>(s[i + 2]);
        const unsigned char c3 = static_cast<unsigned char>(s[i + 3]);

        const int v0 = val(c0);
        const int v1 = val(c1);
        if (v0 < 0 || v1 < 0) {
            fail(daemon_mode, "Error: invalid base64 character");
        }

        if (c2 == '=') {
            // Only legal as '==', and only in the final quantum.
            if (c3 != '=' || i + 4 != s.size()) {
                fail(daemon_mode, "Error: invalid base64 padding");
            }
            out.push_back(static_cast<u8>((v0 << 2) | (v1 >> 4)));
            break;
        }

        const int v2 = val(c2);
        if (v2 < 0) {
            fail(daemon_mode, "Error: invalid base64 character");
        }

        if (c3 == '=') {
            // Only legal in the final quantum.
            if (i + 4 != s.size()) {
                fail(daemon_mode, "Error: invalid base64 padding");
            }
            out.push_back(static_cast<u8>((v0 << 2) | (v1 >> 4)));
            out.push_back(static_cast<u8>(((v1 & 0x0f) << 4) | (v2 >> 2)));
            break;
        }

        const int v3 = val(c3);
        if (v3 < 0) {
            fail(daemon_mode, "Error: invalid base64 character");
        }

        out.push_back(static_cast<u8>((v0 << 2) | (v1 >> 4)));
        out.push_back(static_cast<u8>(((v1 & 0x0f) << 4) | (v2 >> 2)));
        out.push_back(static_cast<u8>(((v2 & 0x03) << 6) | v3));
    }

    return out;
}

struct NullBuffer final : public std::streambuf {
    using int_type = std::streambuf::int_type;
    int_type overflow(int_type ch) override { return traits_type::not_eof(ch); }
    int sync() override { return 0; }
};

#if !defined(_WIN32)
struct ScopedStdoutFdNull final {
    int saved_fd = -1;

    explicit ScopedStdoutFdNull(bool daemon_mode) {
        // In daemon mode, stdout is reserved for the JSON protocol. Redirect FD 1 to /dev/null
        // while running the verifier so *any* stdout writes (printf, puts, library logs) do not
        // corrupt the protocol stream.
        std::fflush(stdout);
        saved_fd = ::dup(1);
        if (saved_fd < 0) {
            fail(daemon_mode, "Error: failed to dup stdout fd");
        }

        const int null_fd = ::open("/dev/null", O_WRONLY);
        if (null_fd < 0) {
            ::close(saved_fd);
            saved_fd = -1;
            fail(daemon_mode, "Error: failed to open /dev/null for stdout redirection");
        }

        if (::dup2(null_fd, 1) < 0) {
            ::close(null_fd);
            ::close(saved_fd);
            saved_fd = -1;
            fail(daemon_mode, "Error: failed to redirect stdout fd to /dev/null");
        }

        ::close(null_fd);
    }

    ScopedStdoutFdNull(const ScopedStdoutFdNull&) = delete;
    ScopedStdoutFdNull& operator=(const ScopedStdoutFdNull&) = delete;

    ~ScopedStdoutFdNull() {
        if (saved_fd >= 0) {
            std::fflush(stdout);
            ::dup2(saved_fd, 1);
            ::close(saved_fd);
            saved_fd = -1;
        }
    }
};
#endif

struct ScopedCoutSilence final {
    std::ostream& out;
    std::streambuf* saved_buf;

    explicit ScopedCoutSilence(std::ostream& out, std::streambuf* new_buf)
        : out(out), saved_buf(out.rdbuf(new_buf)) {
        out.clear();
    }

    ScopedCoutSilence(const ScopedCoutSilence&) = delete;
    ScopedCoutSilence& operator=(const ScopedCoutSilence&) = delete;

    ~ScopedCoutSilence() {
        out.rdbuf(saved_buf);
        out.clear();
    }
};

struct VerifierDaemonCache {
    // Cache WebGPU init across daemon requests.
    std::unique_ptr<executor_t> executor;
    size_t k = 0;
    size_t l = 0;
    size_t n = 0;
    size_t gpu_threads = 0;
    std::string shader_path;

    // Cache parsed WASM module across daemon requests when `program` is unchanged.
    std::unique_ptr<wabt::Module> module;
    fs::path program_path;
    std::optional<fs::file_time_type> program_mtime;
};

static std::unique_ptr<wabt::Module> parse_wasm_module_or_fail(const fs::path& program_name, bool daemon_mode) {
    std::unique_ptr<wabt::Module> wabt_module{ new wabt::Module{} };

    std::vector<uint8_t> program_data;
    wabt::Result read_result = wabt::ReadFile(program_name.c_str(), &program_data);
    if (wabt::Failed(read_result)) {
        fail(daemon_mode,
             std::format("Error: Could not read from file \"{}\"",
                         program_name.c_str()));
    }

    wabt::Features wabt_features;
    wabt::Result   parsing_result;
    wabt::Errors   parsing_errors;
    if (program_name.extension() == ".wat" || program_name.extension() == ".wast") {
        std::unique_ptr<wabt::WastLexer> lexer = wabt::WastLexer::CreateBufferLexer(
            program_name.c_str(),
            program_data.data(),
            program_data.size(),
            &parsing_errors);

        wabt::WastParseOptions parse_wast_options(wabt_features);
        parsing_result = wabt::ParseWatModule(lexer.get(),
                                              &wabt_module,
                                              &parsing_errors,
                                              &parse_wast_options);
    } else {
        parsing_result = wabt::ReadBinaryIr(program_name.c_str(),
                                            program_data.data(),
                                            program_data.size(),
                                            wabt::ReadBinaryOptions{},
                                            &parsing_errors,
                                            wabt_module.get());
    }

    if (wabt::Failed(parsing_result)) {
        auto err_msg = wabt::FormatErrorsToString(parsing_errors,
                                                  wabt::Location::Type::Binary);
        fail(daemon_mode,
             std::format("wabt: {}Error: Failed to parse WASM module \"{}\"",
                         err_msg, program_name.c_str()));
    }

    return wabt_module;
}

static void ensure_cached_module(VerifierDaemonCache& cache, const fs::path& program_name, bool daemon_mode) {
    std::optional<fs::file_time_type> mt;
    try {
        mt = fs::last_write_time(program_name);
    } catch (...) {
        // Ignore mtime failures; we can still cache by path.
    }

    const bool same_path = (!cache.program_path.empty() && cache.program_path == program_name);
    if (cache.module && same_path) {
        // If we can read an mtime now, require that it matches the cached one; if the cached mtime is
        // missing but a current mtime is available, reload once to establish a reliable baseline.
        if (mt.has_value()) {
            if (cache.program_mtime.has_value() && *cache.program_mtime == *mt) {
                return;
            }
        } else {
            // No current mtime: best-effort reuse by path.
            return;
        }
    }

    cache.module = parse_wasm_module_or_fail(program_name, daemon_mode);
    cache.program_path = program_name;
    cache.program_mtime = mt;
    if (daemon_mode) {
        std::cerr << "[daemon] cached wasm module: " << program_name << std::endl;
    }
}

static void ensure_cached_executor(VerifierDaemonCache& cache,
                                   size_t k,
                                   size_t l,
                                   size_t n,
                                   size_t gpu_threads,
                                   const std::string& shader_path,
                                   bool daemon_mode) {
    const bool can_reuse =
        cache.executor &&
        cache.k == k &&
        cache.l == l &&
        cache.n == n &&
        cache.gpu_threads == gpu_threads &&
        cache.shader_path == shader_path;

    if (can_reuse) {
        return;
    }

    auto [omega_k, omega_2k, omega_4k] = field_t::generate_omegas(k, n);

    cache.executor = std::make_unique<executor_t>();
    cache.executor->webgpu_init(gpu_threads, shader_path);
    cache.executor->ntt_init(l, k, n,
                             field_t::modulus, field_t::barrett_factor,
                             omega_k, omega_2k, omega_4k);

    cache.k = k;
    cache.l = l;
    cache.n = n;
    cache.gpu_threads = gpu_threads;
    cache.shader_path = shader_path;

    if (daemon_mode) {
        std::cerr << "[daemon] initialized webgpu executor (gpu_threads=" << gpu_threads
                  << " k=" << k << " l=" << l << " n=" << n
                  << " shader_path=" << shader_path << ")" << std::endl;
    }
}

int run_verifier_from_config(const json& jconfig, bool daemon_mode, VerifierDaemonCache* daemon_cache) {
    const std::string ligero_version_string =
        std::format("ligero-prover v{}.{}.{}+{}.{}",
                    LIGETRON_VERSION_MAJOR,
                    LIGETRON_VERSION_MINOR,
                    LIGETRON_VERSION_PATCH,
                    LIGETRON_GIT_BRANCH,
                    LIGETRON_GIT_COMMIT_HASH);
    if (!daemon_mode) {
        std::cout << ligero_version_string << std::endl;
    }

    std::vector<std::vector<u8>> input_args;
    std::string shader_path;

    bool gzip_proof = true;
    bool gzip_proof_overridden = false;
    if (jconfig.contains("gzip-proof")) {
        gzip_proof = jconfig["gzip-proof"].template get<bool>();
        gzip_proof_overridden = true;
    }

    std::string proof_name = gzip_proof ? "proof_data.gz" : "proof_data.bin";
    if (jconfig.contains("proof-path")) {
        proof_name = jconfig["proof-path"].template get<std::string>();
    }

    size_t k = params::default_row_size;
    size_t l = params::default_packing_size;
    size_t n = params::default_encoding_size;
    
    if (jconfig.contains("packing")) {
        uint64_t packing = jconfig["packing"];
        k = packing;
        l = k - params::sample_size;
        n = 4 * k;
    }
    std::cout << "packing: " << l << ", padding: " << k << ", encoding: " << n << std::endl;

    if (jconfig.contains("shader-path")) {
        shader_path = jconfig["shader-path"].template get<std::string>();
    }

    size_t gpu_threads = k;
    if (jconfig.contains("gpu-threads")) {
        gpu_threads = jconfig["gpu-threads"].template get<size_t>();
    }

    {
        const std::string arg0("Ligero");
        input_args.emplace_back((u8*)arg0.c_str(), (u8*)arg0.c_str() + arg0.size() + 1);
    }

    // NOTE [byte args: `hex` and `bytes_b64`]
    //
    // JSON configs cannot carry raw bytes directly, so we encode byte strings as either:
    // - `{"hex":"..."}`      (hex string, optionally `0x`-prefixed), or
    // - `{"bytes_b64":"..."}` (base64 string).
    //
    // Both are decoded here and passed to the WASM guest as raw bytes (no terminator). Guests should
    // use `args.get_as_bytes()` to read them.
    //
    // Rationale: decoding on the host avoids expensive per-proof parsing inside the WASM guest
    // (especially when many 32-byte values are passed). Prefer `bytes_b64` for large blobs since it
    // is more compact than hex in JSON.
    
    const bool log_args = !daemon_mode && (std::getenv("LIGERO_LOG_ARGS") != nullptr);
    if (jconfig.contains("args")) {
        size_t arg_idx = 0;
        for (const auto& arg : jconfig["args"]) {
            ++arg_idx;
            
            if (arg.contains("i64")) {
                auto i = arg["i64"].template get<int64_t>();
                if (log_args) {
                    std::cout << "[arg#" << arg_idx << "] i64\n";
                }
                input_args.emplace_back((u8*)&i, (u8*)&i + sizeof(int64_t));
            }
            else if (arg.contains("str")) {
                auto str = arg["str"].template get<std::string>();
                if (log_args) {
                    std::cout << "[arg#" << arg_idx << "] str len=" << str.size() << "\n";
                }
                input_args.emplace_back((u8*)str.c_str(), (u8*)str.c_str() + str.size() + 1);
            }
            else if (arg.contains("bytes_b64")) {
                const std::string b64 = arg["bytes_b64"].template get<std::string>();
                if (log_args) {
                    std::cout << "[arg#" << arg_idx << "] bytes_b64 chars=" << b64.size() << "\n";
                }
                input_args.emplace_back(decode_base64_arg_or_fail(b64, daemon_mode));
            }
            else if (arg.contains("hex")) {
                std::string hex_str = arg["hex"].template get<std::string>();
                if (log_args) {
                    std::cout << "[arg#" << arg_idx << "] hex chars=" << hex_str.size() << "\n";
                }

                // Reject embedded NULs (should not be present in JSON strings, but be safe).
                if (hex_str.find('\0') != std::string::npos) {
                    fail(daemon_mode, "Error: hex arg contains embedded NUL byte");
                }

                // Strip optional 0x prefix.
                if (hex_str.size() >= 2 && hex_str[0] == '0' &&
                    (hex_str[1] == 'x' || hex_str[1] == 'X')) {
                    hex_str.erase(0, 2);
                }

                // Pad odd digit counts with leading 0 for proper byte alignment.
                if (hex_str.size() % 2 == 1) {
                    hex_str.insert(hex_str.begin(), '0');
                }

                // Validate hex characters.
                auto is_hex_digit = [](unsigned char c) { return std::isxdigit(c) != 0; };
                if (!std::all_of(hex_str.begin(), hex_str.end(), is_hex_digit)) {
                    fail(daemon_mode, std::format("Error: invalid hex string: {}", hex_str));
                }

                // Decode hex string to raw bytes (no terminator); guest reads bytes directly.
                std::vector<u8> raw;
                raw.reserve(hex_str.size() / 2);
                try {
                    boost::algorithm::unhex(hex_str.begin(), hex_str.end(),
                                           std::back_inserter(raw));
                } catch (const std::exception& e) {
                    fail(daemon_mode,
                         std::format("Error: failed to decode hex string: {} ({})", hex_str,
                                     e.what()));
                }
                input_args.emplace_back(std::move(raw));
            }
            else {
                fail(daemon_mode, std::format("Invalid args type: {}", arg.dump()));
            }
        }
    }

    std::unordered_set<int> indices_set;
    if (jconfig.contains("private-indices")) {
        indices_set = jconfig["private-indices"].template get<std::unordered_set<int>>();
    }

    fs::path program_name;
    if (jconfig.contains("program")) {
        program_name = jconfig["program"].template get<std::string>();
    }
    
    // Reading / parsing the wasm and initializing WebGPU are expensive.
    // In daemon mode, reuse both across requests when inputs match.
    wabt::Module* wabt_module_ptr = nullptr;
    executor_t* executor_ptr = nullptr;
    std::unique_ptr<wabt::Module> wabt_module_local;
    std::unique_ptr<executor_t> executor_local;

    if (daemon_mode && daemon_cache) {
        ensure_cached_module(*daemon_cache, program_name, daemon_mode);
        ensure_cached_executor(*daemon_cache, k, l, n, gpu_threads, shader_path, daemon_mode);
        wabt_module_ptr = daemon_cache->module.get();
        executor_ptr = daemon_cache->executor.get();
    } else {
        wabt_module_local = parse_wasm_module_or_fail(program_name, daemon_mode);
        auto [omega_k, omega_2k, omega_4k] = field_t::generate_omegas(k, n);
        executor_local = std::make_unique<executor_t>();
        executor_local->webgpu_init(gpu_threads, shader_path);
        executor_local->ntt_init(l, k, n,
                                 field_t::modulus, field_t::barrett_factor,
                                 omega_k, omega_2k, omega_4k);
        wabt_module_ptr = wabt_module_local.get();
        executor_ptr = executor_local.get();
    }

    executor_t& executor = *executor_ptr;

    // ================================================================================

    params::hasher::digest stage1_root;
    params::hasher::digest sample_seed;
    std::vector<uint32_t> encoded_code_limbs, encoded_linear_limbs, encoded_quad_limbs;
    zkp::merkle_tree<params::hasher>::decommitment decommit;

    std::stringstream compressed_proof;
    io::filtering_istream proof_stream;
    std::unique_ptr<portable_binary_iarchive> archive_ptr;
    try {
        std::ifstream proof_file(proof_name, std::ios::in | std::ios::binary);

        if (!proof_file) {
            fail(daemon_mode,
                 std::format("Error: Could not read from file \"{}\"", proof_name));
        }

        compressed_proof << proof_file.rdbuf();
        proof_file.close();

        // If caller didn't specify gzip-proof, auto-detect gzip by magic bytes.
        if (!gzip_proof_overridden) {
            const std::string s = compressed_proof.str();
            const bool is_gzip = s.size() >= 2 &&
                                 (static_cast<unsigned char>(s[0]) == 0x1f) &&
                                 (static_cast<unsigned char>(s[1]) == 0x8b);
            gzip_proof = is_gzip;
        }

        if (gzip_proof) {
            proof_stream.push(io::gzip_decompressor());
        }
        proof_stream.push(compressed_proof);

        archive_ptr = std::make_unique<portable_binary_iarchive>(proof_stream);
        *archive_ptr >> stage1_root
                     >> sample_seed
                     >> encoded_code_limbs
                     >> encoded_linear_limbs
                     >> encoded_quad_limbs
                     >> decommit;
    }
    catch (const boost::archive::archive_exception& ex) {
        switch (ex.code) {
            case boost::archive::archive_exception::unsupported_version:
                std::cerr
                    << "Error: boost.archive: " << ex.what() << std::endl
                    << "It seems the proof was created with a newer version of Boost.Archive. \n"
                    << "Please update your Boost version to latest, or ask the file creator to use an older version." << std::endl;
                break;
            default:
                std::cerr << "Error: boost.archive: " << ex.what() << std::endl;
                break;
        }

        std::cerr << "Verification failed, exiting" << std::endl;
        fail(daemon_mode, "Verification failed (boost.archive exception)");
    }
    catch (const std::exception& ex) {
        fail(daemon_mode, std::string("Error: ") + ex.what());
    }

    std::cout << "=============== Start Verify ===============" << std::endl;

    auto vt = make_timer("Verify time");

    // Prepare random seed
    unsigned char seed[params::hasher::digest_size];
    std::copy(stage1_root.begin(), stage1_root.end(), seed);

    // Re-generate sample indexes
    zkp::hash_random_engine<params::hasher> engine(sample_seed);
    std::vector<size_t> indexes(n), sample_index;
    std::iota(indexes.begin(), indexes.end(), 0);
    portable_sample(indexes.begin(), indexes.end(),
                    std::back_inserter(sample_index),
                    params::sample_size,
                    engine);
    std::sort(sample_index.begin(), sample_index.end());
    
    auto vctx = std::make_unique<
        zkp::nonbatch_verifier_context<field_t,
                                       executor_t,
                                       zkp::verifier_random_policy,
                                       params::hasher,
                                       portable_binary_iarchive>>(executor,
                                                                  sample_index,
                                                                  *archive_ptr);
    vctx-> init_witness_random(seed, params::any_iv);

    try {
        run_program(*wabt_module_ptr, *vctx, input_args, indices_set);

        if (proof_stream.peek() != EOF) {
            fail(daemon_mode, "Error: proof size is bigger than it should be");
        }
    }
    catch (const std::exception& e) {
        fail(daemon_mode, std::string("Error: ") + e.what());
    }

    vt.stop();

    auto vs1_root = zkp::merkle_tree<params::hasher>::recommit(vctx->flush_digests(), decommit);

    // ------------------------------------------------------------
    
    auto linear_sums = vctx->linear_sums();
    buffer_t vcode_buffer   = vctx->code();
    buffer_t vlinear_buffer = vctx->linear();
    buffer_t vquad_buffer   = vctx->quadratic();

    mpz_vector vsample_code, vsample_linear, vsample_quad;

    auto vsample_code_limbs = executor.template copy_to_host<uint32_t>(vcode_buffer);
    vsample_code.import_limbs(vsample_code_limbs.data(),
                              vsample_code_limbs.size(),
                              sizeof(uint32_t),
                              field_t::num_u32_limbs);

    auto vsample_linear_limbs = executor.template copy_to_host<uint32_t>(vlinear_buffer);
    vsample_linear.import_limbs(vsample_linear_limbs.data(),
                                vsample_linear_limbs.size(),
                                sizeof(uint32_t),
                                field_t::num_u32_limbs);

    auto vsample_quad_limbs = executor.template copy_to_host<uint32_t>(vquad_buffer);
    vsample_quad.import_limbs(vsample_quad_limbs.data(),
                              vsample_quad_limbs.size(),
                              sizeof(uint32_t),
                              field_t::num_u32_limbs);

    // --------------------------------------------------

    buffer_t device_code   = executor.make_codeword_buffer();
    buffer_t device_linear = executor.make_codeword_buffer();
    buffer_t device_quad   = executor.make_codeword_buffer();

    executor.write_buffer(device_code,   encoded_code_limbs.data(),   encoded_code_limbs.size());
    executor.write_buffer(device_linear, encoded_linear_limbs.data(), encoded_linear_limbs.size());
    executor.write_buffer(device_quad,   encoded_quad_limbs.data(),   encoded_quad_limbs.size());

    auto bind_ntt_pc = executor.bind_ntt(device_code);
    auto bind_ntt_pl = executor.bind_ntt(device_linear);
    auto bind_ntt_pq = executor.bind_ntt(device_quad);

    executor.decode_ntt_device(bind_ntt_pc);
    executor.decode_ntt_device(bind_ntt_pl);
    executor.decode_ntt_device(bind_ntt_pq);
    
    mpz_vector prover_code, prover_linear, prover_quad;

    {
        auto limbs = executor.template copy_to_host<uint32_t>(device_code);
        prover_code.import_limbs(limbs.data(),
                                 limbs.size(),
                                 sizeof(uint32_t),
                                 field_t::num_u32_limbs);
    }
    {
        auto limbs = executor.template copy_to_host<uint32_t>(device_linear);
        prover_linear.import_limbs(limbs.data(),
                                   limbs.size(),
                                   sizeof(uint32_t),
                                   field_t::num_u32_limbs);
        prover_linear.resize(l);
    }
    {
        auto limbs = executor.template copy_to_host<uint32_t>(device_quad);
        prover_quad.import_limbs(limbs.data(),
                                 limbs.size(),
                                 sizeof(uint32_t),
                                 field_t::num_u32_limbs);
        prover_quad.resize(l);
    }


    mpz_vector prover_encoded_codes, prover_encoded_linears, prover_encoded_quads;
    prover_encoded_codes.import_limbs(encoded_code_limbs.data(),
                                      encoded_code_limbs.size(),
                                      sizeof(uint32_t),
                                      field_t::num_u32_limbs);
    prover_encoded_linears.import_limbs(encoded_linear_limbs.data(),
                                        encoded_linear_limbs.size(),
                                        sizeof(uint32_t),
                                        field_t::num_u32_limbs);
    prover_encoded_quads.import_limbs(encoded_quad_limbs.data(),
                                      encoded_quad_limbs.size(),
                                      sizeof(uint32_t),
                                      field_t::num_u32_limbs);

    std::cout << std::boolalpha;
    
    bool valid_merkle = stage1_root == vs1_root;
    bool valid_code   = std::all_of(prover_code.begin() + k, prover_code.end(),
                                  [](const auto& x) { return x == 0; });
    bool valid_linear = zkp::validate_sum<field_t>(prover_linear, linear_sums);
    bool valid_quad   = zkp::validate(prover_quad);

    std::cout << std::endl;
    std::cout << "Prover root  : ";
    zkp::show_hash(stage1_root);
    std::cout << "Verifier root: ";
    zkp::show_hash(vs1_root);
    std::cout << "Validating Merkle Tree Root:         "
              << valid_merkle   << std::endl;
    std::cout << "Validating Encoding Correctness:     "
              << valid_code   << std::endl;
    std::cout << "Validating Linear Constraints:       ";
    std::cout << valid_linear << " " << std::endl;
    std::cout << "Validating Quadratic Constraints:    ";
    std::cout << valid_quad << " " << std::endl;

    bool code_equal = true, linear_equal = true, quad_equal = true;
    for (size_t i = 0; i < params::sample_size; i++) {
        code_equal   &= prover_encoded_codes[sample_index[i]]   == vsample_code[i];
        linear_equal &= prover_encoded_linears[sample_index[i]] == vsample_linear[i];
        quad_equal   &= prover_encoded_quads[sample_index[i]]   == vsample_quad[i];
    }

    bool verify_result = valid_merkle &&
        valid_code && valid_linear && valid_quad &&
        code_equal && linear_equal && quad_equal;

    std::cout << "Validating Encoding Equality:        " << code_equal   << std::endl
              << "Validating Linear Equality:          " << linear_equal << std::endl
              << "Validating Quadratic Equality:       " << quad_equal   << std::endl
              << "-----------------------------------------" << std::endl
              << "Final Verify Result:                 " << verify_result << std::endl;
    
    show_timer();

#if defined(__EMSCRIPTEN__)
    // Since wasm remains loaded in memory after the "main" invocation,
    // timers are not cleaned up before the next invocation,
    // therefore we need to do it manually.
    clear_timers();
#endif
    
    return !verify_result;
}

} // namespace

int main(int argc, const char *argv[]) {
    const bool daemon_mode = (argc >= 2 && std::string_view(argv[1]) == "--daemon");

    if (daemon_mode) {
        // Protocol: one JSON request per line on stdin, one JSON response per line on stdout.
        // stdout is reserved for the protocol, so silence normal informational logs while verifying.
        NullBuffer null_buf;
        std::ostream null_out(&null_buf);

        VerifierDaemonCache cache;

        std::string line;
        while (std::getline(std::cin, line)) {
            json resp;
            try {
                json jconfig = json::parse(line);
                if (jconfig.contains("id")) {
                    resp["id"] = jconfig["id"];
                }

                int exit_code = 0;
                {
#if !defined(_WIN32)
                    ScopedStdoutFdNull fd_null(/*daemon_mode=*/true);
#endif
                    ScopedCoutSilence silence(std::cout, null_out.rdbuf());
                    exit_code = run_verifier_from_config(jconfig, /*daemon_mode=*/true, &cache);
                }
                resp["ok"] = (exit_code == 0);
                resp["exit_code"] = exit_code;
                resp["verify_ok"] = (exit_code == 0);
            } catch (const json::exception& e) {
                resp["ok"] = false;
                resp["error"] = std::string("json parse error: ") + e.what();
            } catch (const DaemonError& e) {
                resp["ok"] = false;
                resp["error"] = e.what();
            } catch (const std::exception& e) {
                resp["ok"] = false;
                resp["error"] = e.what();
            }

            std::cout.clear();
            std::cout << resp.dump() << "\n" << std::flush;
        }
        return 0;
    }

    if (argc < 2) {
        std::cerr << "Error: No JSON input provided" << std::endl;
        return EXIT_FAILURE;
    }

    json jconfig;
    try {
        jconfig = json::parse(std::string_view(argv[1]));
    } catch (json::exception& e) {
        std::cerr << e.what() << std::endl;
        return EXIT_FAILURE;
    }

    return run_verifier_from_config(jconfig, /*daemon_mode=*/false, /*daemon_cache=*/nullptr);
}
