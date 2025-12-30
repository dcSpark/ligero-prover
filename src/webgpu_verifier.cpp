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
#include <filesystem>
#include <fstream>
#include <iostream>
#include <memory>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <unordered_set>

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

struct NullBuffer final : public std::streambuf {
    int overflow(int c) override { return c; }
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
    const bool same_mtime =
        same_path && cache.program_mtime.has_value() && mt.has_value() &&
        *cache.program_mtime == *mt;

    if (cache.module && same_path && (same_mtime || !mt.has_value() || !cache.program_mtime.has_value())) {
        return;
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
    std::string proof_name = "proof_data.gz";
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
    
    if (jconfig.contains("args")) {
        for (const auto& arg : jconfig["args"]) {
            std::cout << "args: " << arg.dump() << std::endl;
            
            if (arg.contains("i64")) {
                auto i = arg["i64"].template get<int64_t>();
                input_args.emplace_back((u8*)&i, (u8*)&i + sizeof(int64_t));
            }
            else if (arg.contains("str")) {
                auto str = arg["str"].template get<std::string>();
                input_args.emplace_back((u8*)str.c_str(), (u8*)str.c_str() + str.size() + 1);
            }
            else if (arg.contains("hex")) {
                // Pass hex as ASCII string (not decoded bytes) - guest parses it
                std::string hex_str = arg["hex"].template get<std::string>();

                // Ensure no embedded NULs (would truncate if guest uses C-string ops)
                if (hex_str.find('\0') != std::string::npos) {
                    fail(daemon_mode, "Error: hex arg contains embedded NUL byte");
                }

                // Ensure 0x prefix (guest/SDK expects it for base-0 parsing)
                if (!(hex_str.size() >= 2 && hex_str[0] == '0' &&
                      (hex_str[1] == 'x' || hex_str[1] == 'X'))) {
                    hex_str = "0x" + hex_str;
                } else if (hex_str[1] == 'X') {
                    hex_str[1] = 'x';  // normalize to lowercase
                }

                // Pad odd digit counts with leading 0 (after 0x) for proper byte alignment
                const size_t digit_count = hex_str.size() - 2;
                if (digit_count % 2 == 1) {
                    hex_str.insert(hex_str.begin() + 2, '0');
                }

                // Validate hex characters
                auto is_hex_digit = [](unsigned char c) { return std::isxdigit(c) != 0; };
                if (!std::all_of(hex_str.begin() + 2, hex_str.end(), is_hex_digit)) {
                    fail(daemon_mode, std::format("Error: invalid hex string: {}", hex_str));
                }

                // Pass as ASCII string with explicit NUL terminator
                std::vector<u8> buf(hex_str.size() + 1);
                std::memcpy(buf.data(), hex_str.data(), hex_str.size());
                buf[hex_str.size()] = 0;

                input_args.emplace_back(std::move(buf));
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

        proof_stream.push(io::gzip_decompressor());
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
        auto* orig_cout_buf = std::cout.rdbuf();

        VerifierDaemonCache cache;

        std::string line;
        while (std::getline(std::cin, line)) {
            json resp;
            try {
                json jconfig = json::parse(line);
                if (jconfig.contains("id")) {
                    resp["id"] = jconfig["id"];
                }

                std::cout.rdbuf(null_out.rdbuf());
                int exit_code = run_verifier_from_config(jconfig, /*daemon_mode=*/true, &cache);
                std::cout.rdbuf(orig_cout_buf);

                resp["ok"] = (exit_code == 0);
                resp["exit_code"] = exit_code;
                resp["verify_ok"] = (exit_code == 0);
            } catch (const json::exception& e) {
                std::cout.rdbuf(orig_cout_buf);
                resp["ok"] = false;
                resp["error"] = std::string("json parse error: ") + e.what();
            } catch (const DaemonError& e) {
                std::cout.rdbuf(orig_cout_buf);
                resp["ok"] = false;
                resp["error"] = e.what();
            } catch (const std::exception& e) {
                std::cout.rdbuf(orig_cout_buf);
                resp["ok"] = false;
                resp["error"] = e.what();
            }

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
