(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func (param i32)))
  (type (;3;) (func (param i32 i32)))
  (type (;4;) (func (param i32 i32 i32)))
  (type (;5;) (func (param i32 i32 i32 i32)))
  (type (;6;) (func (param i32) (result i32)))
  (type (;7;) (func (param i32 i64)))
  (type (;8;) (func))
  (type (;9;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32) (result i32)))
  (type (;11;) (func (result i32)))
  (type (;12;) (func (param i32 i32 i32 i32 i32)))
  (import "wasi_snapshot_preview1" "args_sizes_get" (func (;0;) (type 0)))
  (import "wasi_snapshot_preview1" "args_get" (func (;1;) (type 0)))
  (import "bn254fr" "bn254fr_alloc" (func (;2;) (type 2)))
  (import "bn254fr" "bn254fr_set_u64" (func (;3;) (type 7)))
  (import "bn254fr" "bn254fr_assert_equal_u64" (func (;4;) (type 7)))
  (import "env" "assert_zero" (func (;5;) (type 2)))
  (import "bn254fr" "bn254fr_get_bytes" (func (;6;) (type 5)))
  (import "bn254fr" "bn254fr_free" (func (;7;) (type 2)))
  (import "bn254fr" "bn254fr_assert_equal_bytes" (func (;8;) (type 5)))
  (import "bn254fr" "bn254fr_set_bytes" (func (;9;) (type 5)))
  (import "bn254fr" "bn254fr_set_u32" (func (;10;) (type 3)))
  (import "bn254fr" "bn254fr_submod" (func (;11;) (type 4)))
  (import "bn254fr" "bn254fr_assert_add" (func (;12;) (type 4)))
  (import "bn254fr" "bn254fr_addmod" (func (;13;) (type 4)))
  (import "env" "print_str" (func (;14;) (type 3)))
  (import "bn254fr" "bn254fr_mulmod" (func (;15;) (type 4)))
  (import "bn254fr" "bn254fr_assert_mul" (func (;16;) (type 4)))
  (import "bn254fr" "bn254fr_set_str" (func (;17;) (type 4)))
  (import "bn254fr" "bn254fr_copy" (func (;18;) (type 3)))
  (import "bn254fr" "bn254fr_assert_equal" (func (;19;) (type 3)))
  (import "wasi_snapshot_preview1" "fd_write" (func (;20;) (type 10)))
  (import "wasi_snapshot_preview1" "environ_get" (func (;21;) (type 0)))
  (import "wasi_snapshot_preview1" "environ_sizes_get" (func (;22;) (type 0)))
  (import "wasi_snapshot_preview1" "proc_exit" (func (;23;) (type 2)))
  (func (;24;) (type 8)
    (local i32)
    block  ;; label = @1
      i32.const 1057760
      i32.load
      i32.eqz
      if  ;; label = @2
        i32.const 1057760
        i32.const 1
        i32.store
        call 35
        local.tee 0
        br_if 1 (;@1;)
        return
      end
      unreachable
    end
    local.get 0
    call 103
    unreachable)
  (func (;25;) (type 2) (param i32)
    (local i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    i32.const 1057765
    i32.load8_u
    i32.const 3
    i32.ne
    if  ;; label = @1
      local.get 1
      i32.const 1
      i32.store8 offset=15
      local.get 1
      i32.const 15
      i32.add
      call 36
    end
    local.get 1
    i32.const 16
    i32.add
    global.set 0
    local.get 0
    call 105
    unreachable)
  (func (;26;) (type 8)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64)
    global.get 0
    i32.const 1376
    i32.sub
    local.tee 0
    global.set 0
    local.get 0
    i32.const 0
    i32.store offset=1328
    local.get 0
    i32.const 0
    i32.store offset=208
    local.get 0
    i64.const 4
    i64.store offset=800 align=4
    local.get 0
    i64.const 0
    i64.store offset=792 align=4
    local.get 0
    i64.const 4294967296
    i64.store offset=784 align=4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 0
                      i32.const 1328
                      i32.add
                      local.get 0
                      i32.const 208
                      i32.add
                      call 0
                      if (result i32)  ;; label = @10
                        i32.const 0
                      else
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              block  ;; label = @14
                                block  ;; label = @15
                                  block  ;; label = @16
                                    block  ;; label = @17
                                      local.get 0
                                      i32.load offset=208
                                      local.tee 1
                                      i32.const 0
                                      i32.ge_s
                                      if  ;; label = @18
                                        block  ;; label = @19
                                          local.get 1
                                          i32.eqz
                                          if  ;; label = @20
                                            i32.const 1
                                            local.set 20
                                            br 1 (;@19;)
                                          end
                                          i32.const 1057764
                                          i32.load8_u
                                          drop
                                          local.get 1
                                          i32.const 1
                                          call 99
                                          local.tee 20
                                          i32.eqz
                                          br_if 2 (;@17;)
                                        end
                                        local.get 0
                                        local.get 1
                                        i32.store offset=792
                                        local.get 0
                                        local.get 20
                                        i32.store offset=788
                                        local.get 0
                                        local.get 1
                                        i32.store offset=784
                                        local.get 0
                                        i32.load offset=1328
                                        local.tee 13
                                        i32.const 1073741823
                                        i32.gt_u
                                        br_if 2 (;@16;)
                                        local.get 13
                                        i32.const 2
                                        i32.shl
                                        local.tee 2
                                        i32.const 2147483645
                                        i32.ge_u
                                        br_if 2 (;@16;)
                                        block  ;; label = @19
                                          local.get 2
                                          i32.eqz
                                          if  ;; label = @20
                                            i32.const 4
                                            local.set 1
                                            br 1 (;@19;)
                                          end
                                          i32.const 1057764
                                          i32.load8_u
                                          drop
                                          local.get 13
                                          local.set 7
                                          local.get 2
                                          call 97
                                          local.tee 1
                                          i32.eqz
                                          br_if 4 (;@15;)
                                        end
                                        block  ;; label = @19
                                          block  ;; label = @20
                                            local.get 13
                                            i32.const 2
                                            i32.ge_u
                                            if  ;; label = @21
                                              local.get 2
                                              i32.const 4
                                              i32.sub
                                              local.tee 2
                                              if  ;; label = @22
                                                local.get 1
                                                i32.const 0
                                                local.get 2
                                                memory.fill
                                              end
                                              local.get 1
                                              local.get 2
                                              i32.add
                                              local.set 2
                                              br 1 (;@20;)
                                            end
                                            local.get 1
                                            local.set 2
                                            local.get 13
                                            i32.eqz
                                            br_if 1 (;@19;)
                                          end
                                          local.get 2
                                          i32.const 0
                                          i32.store
                                          local.get 0
                                          i32.load offset=788
                                          local.set 20
                                        end
                                        local.get 1
                                        local.get 20
                                        call 1
                                        br_if 7 (;@11;)
                                        local.get 0
                                        i32.load offset=1328
                                        local.tee 6
                                        i32.const 536870911
                                        i32.gt_u
                                        br_if 4 (;@14;)
                                        local.get 6
                                        i32.const 3
                                        i32.shl
                                        local.tee 2
                                        i32.const 2147483645
                                        i32.ge_u
                                        br_if 4 (;@14;)
                                        block  ;; label = @19
                                          local.get 2
                                          i32.eqz
                                          if  ;; label = @20
                                            i32.const 4
                                            local.set 16
                                            br 1 (;@19;)
                                          end
                                          i32.const 1057764
                                          i32.load8_u
                                          drop
                                          local.get 6
                                          local.set 3
                                          local.get 2
                                          call 97
                                          local.tee 16
                                          i32.eqz
                                          br_if 6 (;@13;)
                                        end
                                        local.get 0
                                        i32.load offset=796
                                        if  ;; label = @19
                                          local.get 0
                                          i32.load offset=800
                                          call 98
                                          local.get 0
                                          i32.load offset=1328
                                          local.set 6
                                        end
                                        i32.const 0
                                        local.set 2
                                        local.get 0
                                        i32.const 0
                                        i32.store offset=804
                                        local.get 0
                                        local.get 16
                                        i32.store offset=800
                                        local.get 0
                                        local.get 3
                                        i32.store offset=796
                                        local.get 6
                                        i32.eqz
                                        br_if 7 (;@11;)
                                        local.get 0
                                        i32.const 796
                                        i32.add
                                        local.set 14
                                        i32.const 4
                                        local.set 9
                                        local.get 1
                                        local.set 4
                                        loop  ;; label = @19
                                          block (result i32)  ;; label = @20
                                            block  ;; label = @21
                                              local.get 2
                                              local.get 13
                                              i32.ne
                                              if  ;; label = @22
                                                local.get 2
                                                i32.const 1
                                                i32.add
                                                local.set 5
                                                local.get 4
                                                i32.load
                                                local.set 3
                                                local.get 2
                                                local.get 0
                                                i32.load offset=1328
                                                i32.const 1
                                                i32.sub
                                                i32.ne
                                                br_if 1 (;@21;)
                                                local.get 1
                                                i32.load
                                                local.get 0
                                                i32.load offset=208
                                                i32.add
                                                br 2 (;@20;)
                                              end
                                              local.get 13
                                              local.get 13
                                              i32.const 1049536
                                              call 27
                                              unreachable
                                            end
                                            local.get 5
                                            local.get 13
                                            i32.ge_u
                                            br_if 8 (;@12;)
                                            local.get 4
                                            i32.const 4
                                            i32.add
                                            i32.load
                                          end
                                          local.get 3
                                          local.get 20
                                          i32.sub
                                          local.set 22
                                          local.get 20
                                          i32.sub
                                          local.set 23
                                          local.get 0
                                          i32.load offset=796
                                          local.get 2
                                          i32.eq
                                          if  ;; label = @20
                                            global.get 0
                                            i32.const 32
                                            i32.sub
                                            local.tee 2
                                            global.set 0
                                            block  ;; label = @21
                                              block  ;; label = @22
                                                local.get 14
                                                i32.load
                                                local.tee 11
                                                i32.const 268435455
                                                i32.gt_u
                                                br_if 0 (;@22;)
                                                i32.const 4
                                                local.get 11
                                                i32.const 1
                                                i32.shl
                                                local.tee 3
                                                local.get 3
                                                i32.const 4
                                                i32.le_u
                                                select
                                                local.tee 24
                                                i32.const 3
                                                i32.shl
                                                local.tee 3
                                                i32.const 2147483644
                                                i32.gt_u
                                                br_if 0 (;@22;)
                                                local.get 2
                                                local.get 11
                                                if (result i32)  ;; label = @23
                                                  local.get 2
                                                  local.get 11
                                                  i32.const 3
                                                  i32.shl
                                                  i32.store offset=28
                                                  local.get 2
                                                  local.get 14
                                                  i32.load offset=4
                                                  i32.store offset=20
                                                  i32.const 4
                                                else
                                                  i32.const 0
                                                end
                                                i32.store offset=24
                                                local.get 2
                                                i32.const 8
                                                i32.add
                                                local.set 17
                                                local.get 3
                                                local.set 10
                                                i32.const 0
                                                local.set 16
                                                global.get 0
                                                i32.const 16
                                                i32.sub
                                                local.tee 8
                                                global.set 0
                                                block  ;; label = @23
                                                  block  ;; label = @24
                                                    block (result i32)  ;; label = @25
                                                      block  ;; label = @26
                                                        local.get 2
                                                        i32.const 20
                                                        i32.add
                                                        local.tee 3
                                                        i32.load offset=4
                                                        if  ;; label = @27
                                                          local.get 3
                                                          i32.load offset=8
                                                          local.tee 15
                                                          i32.eqz
                                                          if  ;; label = @28
                                                            local.get 10
                                                            i32.eqz
                                                            br_if 4 (;@24;)
                                                            i32.const 1057764
                                                            i32.load8_u
                                                            drop
                                                            local.get 10
                                                            i32.const 3
                                                            i32.gt_u
                                                            br_if 2 (;@26;)
                                                            local.get 8
                                                            i32.const 0
                                                            i32.store offset=4
                                                            i32.const 4
                                                            local.set 11
                                                            local.get 8
                                                            i32.const 4
                                                            i32.add
                                                            local.get 10
                                                            call 102
                                                            i32.eqz
                                                            if  ;; label = @29
                                                              local.get 8
                                                              i32.load offset=4
                                                              br 4 (;@25;)
                                                            end
                                                            i32.const 1
                                                            local.set 16
                                                            br 5 (;@23;)
                                                          end
                                                          local.get 3
                                                          i32.load
                                                          local.set 18
                                                          local.get 10
                                                          i32.const 3
                                                          i32.le_u
                                                          if  ;; label = @28
                                                            local.get 8
                                                            i32.const 0
                                                            i32.store offset=8
                                                            i32.const 4
                                                            local.set 11
                                                            i32.const 1
                                                            local.set 16
                                                            local.get 8
                                                            i32.const 8
                                                            i32.add
                                                            local.get 10
                                                            call 102
                                                            br_if 5 (;@23;)
                                                            local.get 8
                                                            i32.load offset=8
                                                            local.tee 3
                                                            i32.eqz
                                                            br_if 5 (;@23;)
                                                            local.get 15
                                                            if  ;; label = @29
                                                              local.get 3
                                                              local.get 18
                                                              local.get 15
                                                              memory.copy
                                                            end
                                                            local.get 18
                                                            call 98
                                                            i32.const 0
                                                            local.set 16
                                                            local.get 3
                                                            local.set 11
                                                            br 5 (;@23;)
                                                          end
                                                          local.get 18
                                                          local.get 10
                                                          call 100
                                                          br 2 (;@25;)
                                                        end
                                                        local.get 10
                                                        i32.eqz
                                                        br_if 2 (;@24;)
                                                        i32.const 1057764
                                                        i32.load8_u
                                                        drop
                                                        local.get 10
                                                        i32.const 3
                                                        i32.gt_u
                                                        br_if 0 (;@26;)
                                                        local.get 8
                                                        i32.const 0
                                                        i32.store offset=12
                                                        i32.const 4
                                                        local.set 11
                                                        local.get 8
                                                        i32.const 12
                                                        i32.add
                                                        local.get 10
                                                        call 102
                                                        i32.eqz
                                                        if  ;; label = @27
                                                          local.get 8
                                                          i32.load offset=12
                                                          br 2 (;@25;)
                                                        end
                                                        i32.const 1
                                                        local.set 16
                                                        br 3 (;@23;)
                                                      end
                                                      local.get 10
                                                      call 97
                                                    end
                                                    local.tee 3
                                                    i32.const 4
                                                    local.get 3
                                                    select
                                                    local.set 11
                                                    local.get 3
                                                    i32.eqz
                                                    local.set 16
                                                    br 1 (;@23;)
                                                  end
                                                  i32.const 4
                                                  local.set 11
                                                end
                                                local.get 17
                                                local.get 10
                                                i32.store offset=8
                                                local.get 17
                                                local.get 11
                                                i32.store offset=4
                                                local.get 17
                                                local.get 16
                                                i32.store
                                                local.get 8
                                                i32.const 16
                                                i32.add
                                                global.set 0
                                                local.get 2
                                                i32.load offset=8
                                                i32.const 1
                                                i32.ne
                                                br_if 1 (;@21;)
                                                local.get 2
                                                i32.load offset=16
                                                local.set 19
                                                local.get 2
                                                i32.load offset=12
                                                local.set 21
                                              end
                                              local.get 21
                                              local.get 19
                                              i32.const 1049568
                                              call 38
                                              unreachable
                                            end
                                            local.get 2
                                            i32.load offset=12
                                            local.set 3
                                            local.get 14
                                            local.get 24
                                            i32.store
                                            local.get 14
                                            local.get 3
                                            i32.store offset=4
                                            local.get 2
                                            i32.const 32
                                            i32.add
                                            global.set 0
                                            local.get 0
                                            i32.load offset=800
                                            local.set 16
                                          end
                                          local.get 9
                                          local.get 16
                                          i32.add
                                          local.tee 2
                                          local.get 23
                                          i32.store
                                          local.get 2
                                          i32.const 4
                                          i32.sub
                                          local.get 22
                                          i32.store
                                          local.get 0
                                          local.get 5
                                          i32.store offset=804
                                          local.get 9
                                          i32.const 8
                                          i32.add
                                          local.set 9
                                          local.get 4
                                          i32.const 4
                                          i32.add
                                          local.set 4
                                          local.get 6
                                          local.get 5
                                          local.tee 2
                                          i32.ne
                                          br_if 0 (;@19;)
                                        end
                                        br 7 (;@11;)
                                      end
                                      i32.const 1049488
                                      call 28
                                      unreachable
                                    end
                                    local.get 1
                                    call 29
                                    unreachable
                                  end
                                  i32.const 1049504
                                  call 28
                                  unreachable
                                end
                                local.get 2
                                call 29
                                unreachable
                              end
                              i32.const 1049520
                              call 28
                              unreachable
                            end
                            local.get 2
                            call 29
                            unreachable
                          end
                          local.get 5
                          local.get 13
                          i32.const 1049552
                          call 27
                          unreachable
                        end
                        local.get 7
                        if  ;; label = @11
                          local.get 1
                          call 98
                        end
                        local.get 0
                        i32.load offset=804
                      end
                      i32.const 37
                      i32.eq
                      if  ;; label = @10
                        block  ;; label = @11
                          local.get 0
                          i32.load offset=800
                          local.tee 12
                          i32.load offset=12
                          local.tee 2
                          local.get 12
                          i32.load offset=8
                          local.tee 3
                          i32.ge_u
                          if  ;; label = @12
                            local.get 0
                            i32.load offset=792
                            local.tee 14
                            local.get 2
                            i32.lt_u
                            br_if 7 (;@5;)
                            local.get 0
                            i32.load offset=796
                            local.set 23
                            local.get 0
                            i32.load offset=788
                            local.set 8
                            local.get 0
                            i32.load offset=784
                            local.set 24
                            local.get 0
                            i32.const 808
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 800
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 792
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            block  ;; label = @13
                              local.get 2
                              local.get 3
                              i32.sub
                              local.tee 1
                              i32.const 66
                              i32.sub
                              i32.const 2
                              i32.ge_u
                              if  ;; label = @14
                                local.get 1
                                i32.const 32
                                i32.eq
                                if  ;; label = @15
                                  local.get 3
                                  local.get 8
                                  i32.add
                                  local.set 2
                                  br 2 (;@13;)
                                end
                                br 6 (;@8;)
                              end
                              local.get 3
                              local.get 8
                              i32.add
                              i32.const 3
                              i32.add
                              local.set 3
                              i32.const 0
                              local.set 4
                              loop  ;; label = @14
                                local.get 3
                                i32.const 1
                                i32.sub
                                i32.load8_u
                                local.tee 6
                                i32.const 9
                                i32.add
                                local.set 7
                                local.get 4
                                local.get 0
                                i32.const 784
                                i32.add
                                local.tee 2
                                i32.add
                                local.get 3
                                i32.load8_u
                                local.tee 5
                                i32.const 48
                                i32.sub
                                local.tee 1
                                i32.const 0
                                local.get 1
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 5
                                i32.const 87
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 5
                                i32.const 55
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 6
                                i32.const 0
                                local.get 6
                                i32.const 48
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                i32.const 4
                                i32.shl
                                i32.or
                                i32.store8
                                local.get 3
                                i32.const 2
                                i32.add
                                local.set 3
                                local.get 4
                                i32.const 1
                                i32.add
                                local.tee 4
                                i32.const 32
                                i32.ne
                                br_if 0 (;@14;)
                              end
                            end
                            local.get 0
                            i32.const 24
                            i32.add
                            local.get 2
                            i32.const 24
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 0
                            i32.const 16
                            i32.add
                            local.get 2
                            i32.const 16
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 0
                            i32.const 8
                            i32.add
                            local.get 2
                            i32.const 8
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 0
                            local.get 2
                            i64.load align=1
                            i64.store
                            local.get 12
                            i32.load offset=20
                            local.tee 2
                            local.get 12
                            i32.load offset=16
                            local.tee 3
                            i32.lt_u
                            br_if 6 (;@6;)
                            local.get 2
                            local.get 14
                            i32.gt_u
                            br_if 7 (;@5;)
                            local.get 2
                            local.get 3
                            i32.sub
                            i32.const 8
                            i32.ne
                            br_if 1 (;@11;)
                            local.get 3
                            local.get 8
                            i32.add
                            i64.load
                            local.tee 28
                            i64.const 0
                            i64.lt_s
                            br_if 8 (;@4;)
                            local.get 28
                            i64.eqz
                            br_if 8 (;@4;)
                            local.get 0
                            i32.const 784
                            i32.add
                            local.tee 2
                            i32.const 8
                            i32.add
                            local.tee 1
                            i32.const 0
                            i32.store8
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            local.get 2
                            call 2
                            local.get 0
                            i32.const 208
                            i32.add
                            local.tee 2
                            i32.const 8
                            i32.add
                            local.tee 3
                            local.get 1
                            i64.load
                            i64.store
                            local.get 0
                            local.get 0
                            i64.load offset=784
                            i64.store offset=208
                            local.get 2
                            call 2
                            local.get 2
                            local.get 28
                            call 3
                            local.get 0
                            i32.const 32
                            i32.add
                            local.tee 2
                            i32.const 8
                            i32.add
                            local.tee 1
                            local.get 3
                            i64.load
                            i64.store
                            local.get 0
                            local.get 0
                            i64.load offset=208
                            i64.store offset=32
                            local.get 2
                            local.get 28
                            call 4
                            local.get 1
                            i32.const 1
                            i32.store8
                            local.get 12
                            i32.load offset=28
                            local.tee 2
                            local.get 12
                            i32.load offset=24
                            local.tee 3
                            i32.lt_u
                            br_if 6 (;@6;)
                            local.get 2
                            local.get 14
                            i32.gt_u
                            br_if 7 (;@5;)
                            local.get 0
                            i32.const 808
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 800
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 792
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            block  ;; label = @13
                              local.get 2
                              local.get 3
                              i32.sub
                              local.tee 1
                              i32.const 66
                              i32.sub
                              i32.const 2
                              i32.ge_u
                              if  ;; label = @14
                                local.get 1
                                i32.const 32
                                i32.eq
                                if  ;; label = @15
                                  local.get 3
                                  local.get 8
                                  i32.add
                                  local.set 2
                                  br 2 (;@13;)
                                end
                                br 6 (;@8;)
                              end
                              local.get 3
                              local.get 8
                              i32.add
                              i32.const 3
                              i32.add
                              local.set 3
                              i32.const 0
                              local.set 4
                              loop  ;; label = @14
                                local.get 3
                                i32.const 1
                                i32.sub
                                i32.load8_u
                                local.tee 6
                                i32.const 9
                                i32.add
                                local.set 7
                                local.get 4
                                local.get 0
                                i32.const 784
                                i32.add
                                local.tee 2
                                i32.add
                                local.get 3
                                i32.load8_u
                                local.tee 5
                                i32.const 48
                                i32.sub
                                local.tee 1
                                i32.const 0
                                local.get 1
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 5
                                i32.const 87
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 5
                                i32.const 55
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 6
                                i32.const 0
                                local.get 6
                                i32.const 48
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                i32.const 4
                                i32.shl
                                i32.or
                                i32.store8
                                local.get 3
                                i32.const 2
                                i32.add
                                local.set 3
                                local.get 4
                                i32.const 1
                                i32.add
                                local.tee 4
                                i32.const 32
                                i32.ne
                                br_if 0 (;@14;)
                              end
                            end
                            local.get 0
                            i32.const 48
                            i32.add
                            local.tee 1
                            i32.const 24
                            i32.add
                            local.get 2
                            i32.const 24
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 1
                            i32.const 16
                            i32.add
                            local.get 2
                            i32.const 16
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 1
                            i32.const 8
                            i32.add
                            local.get 2
                            i32.const 8
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 0
                            local.get 2
                            i64.load align=1
                            i64.store offset=48
                            local.get 12
                            i32.load offset=36
                            local.tee 2
                            local.get 12
                            i32.load offset=32
                            local.tee 3
                            i32.lt_u
                            br_if 6 (;@6;)
                            local.get 2
                            local.get 14
                            i32.gt_u
                            br_if 7 (;@5;)
                            local.get 0
                            i32.const 808
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 800
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 792
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            block  ;; label = @13
                              local.get 2
                              local.get 3
                              i32.sub
                              local.tee 1
                              i32.const 66
                              i32.sub
                              i32.const 2
                              i32.ge_u
                              if  ;; label = @14
                                local.get 1
                                i32.const 32
                                i32.eq
                                if  ;; label = @15
                                  local.get 3
                                  local.get 8
                                  i32.add
                                  local.set 2
                                  br 2 (;@13;)
                                end
                                br 6 (;@8;)
                              end
                              local.get 3
                              local.get 8
                              i32.add
                              i32.const 3
                              i32.add
                              local.set 3
                              i32.const 0
                              local.set 4
                              loop  ;; label = @14
                                local.get 3
                                i32.const 1
                                i32.sub
                                i32.load8_u
                                local.tee 6
                                i32.const 9
                                i32.add
                                local.set 7
                                local.get 4
                                local.get 0
                                i32.const 784
                                i32.add
                                local.tee 2
                                i32.add
                                local.get 3
                                i32.load8_u
                                local.tee 5
                                i32.const 48
                                i32.sub
                                local.tee 1
                                i32.const 0
                                local.get 1
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 5
                                i32.const 87
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 5
                                i32.const 55
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 6
                                i32.const 0
                                local.get 6
                                i32.const 48
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                i32.const 4
                                i32.shl
                                i32.or
                                i32.store8
                                local.get 3
                                i32.const 2
                                i32.add
                                local.set 3
                                local.get 4
                                i32.const 1
                                i32.add
                                local.tee 4
                                i32.const 32
                                i32.ne
                                br_if 0 (;@14;)
                              end
                            end
                            local.get 0
                            i32.const 80
                            i32.add
                            local.tee 1
                            i32.const 24
                            i32.add
                            local.get 2
                            i32.const 24
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 1
                            i32.const 16
                            i32.add
                            local.get 2
                            i32.const 16
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 1
                            i32.const 8
                            i32.add
                            local.get 2
                            i32.const 8
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 0
                            local.get 2
                            i64.load align=1
                            i64.store offset=80
                            local.get 12
                            i32.load offset=44
                            local.tee 2
                            local.get 12
                            i32.load offset=40
                            local.tee 3
                            i32.lt_u
                            br_if 6 (;@6;)
                            local.get 2
                            local.get 14
                            i32.gt_u
                            br_if 7 (;@5;)
                            local.get 0
                            i32.const 808
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 800
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 792
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            block  ;; label = @13
                              local.get 2
                              local.get 3
                              i32.sub
                              local.tee 1
                              i32.const 66
                              i32.sub
                              i32.const 2
                              i32.ge_u
                              if  ;; label = @14
                                local.get 1
                                i32.const 32
                                i32.eq
                                if  ;; label = @15
                                  local.get 3
                                  local.get 8
                                  i32.add
                                  local.set 2
                                  br 2 (;@13;)
                                end
                                br 6 (;@8;)
                              end
                              local.get 3
                              local.get 8
                              i32.add
                              i32.const 3
                              i32.add
                              local.set 3
                              i32.const 0
                              local.set 4
                              loop  ;; label = @14
                                local.get 3
                                i32.const 1
                                i32.sub
                                i32.load8_u
                                local.tee 6
                                i32.const 9
                                i32.add
                                local.set 7
                                local.get 4
                                local.get 0
                                i32.const 784
                                i32.add
                                local.tee 2
                                i32.add
                                local.get 3
                                i32.load8_u
                                local.tee 5
                                i32.const 48
                                i32.sub
                                local.tee 1
                                i32.const 0
                                local.get 1
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 5
                                i32.const 87
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 5
                                i32.const 55
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 6
                                i32.const 0
                                local.get 6
                                i32.const 48
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 7
                                i32.const 0
                                local.get 6
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                i32.const 4
                                i32.shl
                                i32.or
                                i32.store8
                                local.get 3
                                i32.const 2
                                i32.add
                                local.set 3
                                local.get 4
                                i32.const 1
                                i32.add
                                local.tee 4
                                i32.const 32
                                i32.ne
                                br_if 0 (;@14;)
                              end
                            end
                            local.get 0
                            i32.const 112
                            i32.add
                            local.tee 1
                            i32.const 24
                            i32.add
                            local.get 2
                            i32.const 24
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 1
                            i32.const 16
                            i32.add
                            local.get 2
                            i32.const 16
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 1
                            i32.const 8
                            i32.add
                            local.get 2
                            i32.const 8
                            i32.add
                            i64.load align=1
                            i64.store
                            local.get 0
                            local.get 2
                            i64.load align=1
                            i64.store offset=112
                            local.get 12
                            i32.load offset=52
                            local.tee 3
                            local.get 12
                            i32.load offset=48
                            local.tee 4
                            i32.lt_u
                            br_if 9 (;@3;)
                            local.get 3
                            local.get 14
                            i32.gt_u
                            br_if 10 (;@2;)
                            local.get 0
                            i32.const 808
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 800
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 792
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            local.get 4
                            local.get 8
                            i32.add
                            local.set 2
                            block  ;; label = @13
                              local.get 3
                              local.get 4
                              i32.sub
                              local.tee 1
                              i32.const 66
                              i32.sub
                              i32.const 2
                              i32.ge_u
                              if  ;; label = @14
                                local.get 1
                                i32.const 32
                                i32.eq
                                if  ;; label = @15
                                  local.get 2
                                  i32.const 8
                                  i32.add
                                  i64.load align=1
                                  local.set 26
                                  local.get 2
                                  i32.const 16
                                  i32.add
                                  i64.load align=1
                                  local.set 27
                                  local.get 2
                                  i64.load align=1
                                  local.set 25
                                  local.get 0
                                  i32.const 144
                                  i32.add
                                  local.tee 1
                                  i32.const 24
                                  i32.add
                                  local.get 2
                                  i32.const 24
                                  i32.add
                                  i64.load align=1
                                  i64.store
                                  local.get 1
                                  i32.const 16
                                  i32.add
                                  local.get 27
                                  i64.store
                                  local.get 1
                                  i32.const 8
                                  i32.add
                                  local.get 26
                                  i64.store
                                  local.get 0
                                  local.get 25
                                  i64.store offset=144
                                  br 2 (;@13;)
                                end
                                br 6 (;@8;)
                              end
                              local.get 2
                              i32.const 3
                              i32.add
                              local.set 3
                              i32.const 0
                              local.set 4
                              loop  ;; label = @14
                                local.get 3
                                i32.const 1
                                i32.sub
                                i32.load8_u
                                local.tee 6
                                i32.const 9
                                i32.add
                                local.set 2
                                local.get 0
                                i32.const 784
                                i32.add
                                local.tee 7
                                local.get 4
                                i32.add
                                local.get 3
                                i32.load8_u
                                local.tee 5
                                i32.const 48
                                i32.sub
                                local.tee 1
                                i32.const 0
                                local.get 1
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 5
                                i32.const 87
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 5
                                i32.const 55
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 6
                                i32.const 0
                                local.get 6
                                i32.const 48
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 2
                                i32.const 0
                                local.get 6
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 2
                                i32.const 0
                                local.get 6
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                i32.const 4
                                i32.shl
                                i32.or
                                i32.store8
                                local.get 3
                                i32.const 2
                                i32.add
                                local.set 3
                                local.get 4
                                i32.const 1
                                i32.add
                                local.tee 4
                                i32.const 32
                                i32.ne
                                br_if 0 (;@14;)
                              end
                              local.get 0
                              i32.const 144
                              i32.add
                              local.tee 1
                              i32.const 24
                              i32.add
                              local.get 7
                              i32.const 24
                              i32.add
                              i64.load
                              i64.store
                              local.get 1
                              i32.const 16
                              i32.add
                              local.get 7
                              i32.const 16
                              i32.add
                              i64.load
                              i64.store
                              local.get 1
                              i32.const 8
                              i32.add
                              local.get 7
                              i32.const 8
                              i32.add
                              i64.load
                              i64.store
                              local.get 0
                              local.get 0
                              i64.load offset=784
                              i64.store offset=144
                            end
                            local.get 12
                            i32.load offset=60
                            local.tee 3
                            local.get 12
                            i32.load offset=56
                            local.tee 4
                            i32.lt_u
                            br_if 9 (;@3;)
                            local.get 3
                            local.get 14
                            i32.gt_u
                            br_if 10 (;@2;)
                            local.get 0
                            i32.const 808
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 800
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i32.const 792
                            i32.add
                            i64.const 0
                            i64.store
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            local.get 4
                            local.get 8
                            i32.add
                            local.set 2
                            block  ;; label = @13
                              local.get 3
                              local.get 4
                              i32.sub
                              local.tee 1
                              i32.const 66
                              i32.sub
                              i32.const 2
                              i32.ge_u
                              if  ;; label = @14
                                local.get 1
                                i32.const 32
                                i32.eq
                                if  ;; label = @15
                                  local.get 2
                                  i32.const 8
                                  i32.add
                                  i64.load align=1
                                  local.set 26
                                  local.get 2
                                  i32.const 16
                                  i32.add
                                  i64.load align=1
                                  local.set 27
                                  local.get 2
                                  i64.load align=1
                                  local.set 25
                                  local.get 0
                                  i32.const 176
                                  i32.add
                                  local.tee 1
                                  i32.const 24
                                  i32.add
                                  local.get 2
                                  i32.const 24
                                  i32.add
                                  i64.load align=1
                                  i64.store
                                  local.get 1
                                  i32.const 16
                                  i32.add
                                  local.get 27
                                  i64.store
                                  local.get 1
                                  i32.const 8
                                  i32.add
                                  local.get 26
                                  i64.store
                                  local.get 0
                                  local.get 25
                                  i64.store offset=176
                                  br 2 (;@13;)
                                end
                                br 6 (;@8;)
                              end
                              local.get 2
                              i32.const 3
                              i32.add
                              local.set 3
                              i32.const 0
                              local.set 4
                              loop  ;; label = @14
                                local.get 3
                                i32.const 1
                                i32.sub
                                i32.load8_u
                                local.tee 6
                                i32.const 9
                                i32.add
                                local.set 2
                                local.get 0
                                i32.const 784
                                i32.add
                                local.tee 7
                                local.get 4
                                i32.add
                                local.get 3
                                i32.load8_u
                                local.tee 5
                                i32.const 48
                                i32.sub
                                local.tee 1
                                i32.const 0
                                local.get 1
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 5
                                i32.const 87
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 5
                                i32.const 55
                                i32.sub
                                i32.const 0
                                local.get 5
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 6
                                i32.const 0
                                local.get 6
                                i32.const 48
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 10
                                i32.lt_u
                                select
                                local.get 2
                                i32.const 0
                                local.get 6
                                i32.const 97
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                local.get 2
                                i32.const 0
                                local.get 6
                                i32.const 65
                                i32.sub
                                i32.const 255
                                i32.and
                                i32.const 6
                                i32.lt_u
                                select
                                i32.or
                                i32.const 4
                                i32.shl
                                i32.or
                                i32.store8
                                local.get 3
                                i32.const 2
                                i32.add
                                local.set 3
                                local.get 4
                                i32.const 1
                                i32.add
                                local.tee 4
                                i32.const 32
                                i32.ne
                                br_if 0 (;@14;)
                              end
                              local.get 0
                              i32.const 176
                              i32.add
                              local.tee 1
                              i32.const 24
                              i32.add
                              local.get 7
                              i32.const 24
                              i32.add
                              i64.load
                              i64.store
                              local.get 1
                              i32.const 16
                              i32.add
                              local.get 7
                              i32.const 16
                              i32.add
                              i64.load
                              i64.store
                              local.get 1
                              i32.const 8
                              i32.add
                              local.get 7
                              i32.const 8
                              i32.add
                              i64.load
                              i64.store
                              local.get 0
                              local.get 0
                              i64.load offset=784
                              i64.store offset=176
                            end
                            i32.const 0
                            local.set 20
                            local.get 0
                            i32.const 208
                            i32.add
                            i32.const 0
                            i32.const 384
                            memory.fill
                            local.get 8
                            i32.const 3
                            i32.add
                            local.set 11
                            local.get 0
                            i32.const 784
                            i32.add
                            local.tee 1
                            i32.const 24
                            i32.add
                            local.set 6
                            local.get 1
                            i32.const 16
                            i32.add
                            local.set 5
                            local.get 1
                            i32.const 8
                            i32.add
                            local.set 7
                            loop  ;; label = @13
                              local.get 12
                              local.get 20
                              i32.const 3
                              i32.shl
                              i32.add
                              local.tee 1
                              i32.load offset=68
                              local.tee 2
                              local.get 1
                              i32.load offset=64
                              local.tee 3
                              i32.lt_u
                              br_if 7 (;@6;)
                              local.get 2
                              local.get 14
                              i32.gt_u
                              br_if 8 (;@5;)
                              local.get 6
                              i64.const 0
                              i64.store
                              local.get 5
                              i64.const 0
                              i64.store
                              local.get 7
                              i64.const 0
                              i64.store
                              local.get 0
                              i64.const 0
                              i64.store offset=784
                              block  ;; label = @14
                                local.get 2
                                local.get 3
                                i32.sub
                                local.tee 1
                                i32.const 66
                                i32.sub
                                i32.const 2
                                i32.ge_u
                                if  ;; label = @15
                                  local.get 1
                                  i32.const 32
                                  i32.eq
                                  if  ;; label = @16
                                    local.get 3
                                    local.get 8
                                    i32.add
                                    local.set 2
                                    br 2 (;@14;)
                                  end
                                  br 8 (;@7;)
                                end
                                local.get 3
                                local.get 11
                                i32.add
                                local.set 3
                                i32.const 0
                                local.set 4
                                loop  ;; label = @15
                                  local.get 3
                                  i32.const 1
                                  i32.sub
                                  i32.load8_u
                                  local.tee 21
                                  i32.const 9
                                  i32.add
                                  local.set 13
                                  local.get 4
                                  local.get 0
                                  i32.const 784
                                  i32.add
                                  local.tee 2
                                  i32.add
                                  local.get 3
                                  i32.load8_u
                                  local.tee 10
                                  i32.const 48
                                  i32.sub
                                  local.tee 1
                                  i32.const 0
                                  local.get 1
                                  i32.const 255
                                  i32.and
                                  i32.const 10
                                  i32.lt_u
                                  select
                                  local.get 10
                                  i32.const 87
                                  i32.sub
                                  i32.const 0
                                  local.get 10
                                  i32.const 97
                                  i32.sub
                                  i32.const 255
                                  i32.and
                                  i32.const 6
                                  i32.lt_u
                                  select
                                  i32.or
                                  local.get 10
                                  i32.const 55
                                  i32.sub
                                  i32.const 0
                                  local.get 10
                                  i32.const 65
                                  i32.sub
                                  i32.const 255
                                  i32.and
                                  i32.const 6
                                  i32.lt_u
                                  select
                                  i32.or
                                  local.get 21
                                  i32.const 0
                                  local.get 21
                                  i32.const 48
                                  i32.sub
                                  i32.const 255
                                  i32.and
                                  i32.const 10
                                  i32.lt_u
                                  select
                                  local.get 13
                                  i32.const 0
                                  local.get 21
                                  i32.const 97
                                  i32.sub
                                  i32.const 255
                                  i32.and
                                  i32.const 6
                                  i32.lt_u
                                  select
                                  i32.or
                                  local.get 13
                                  i32.const 0
                                  local.get 21
                                  i32.const 65
                                  i32.sub
                                  i32.const 255
                                  i32.and
                                  i32.const 6
                                  i32.lt_u
                                  select
                                  i32.or
                                  i32.const 4
                                  i32.shl
                                  i32.or
                                  i32.store8
                                  local.get 3
                                  i32.const 2
                                  i32.add
                                  local.set 3
                                  local.get 4
                                  i32.const 1
                                  i32.add
                                  local.tee 4
                                  i32.const 32
                                  i32.ne
                                  br_if 0 (;@15;)
                                end
                              end
                              local.get 2
                              i32.const 24
                              i32.add
                              i64.load align=1
                              local.set 26
                              local.get 2
                              i32.const 16
                              i32.add
                              i64.load align=1
                              local.set 27
                              local.get 2
                              i32.const 8
                              i32.add
                              i64.load align=1
                              local.set 25
                              local.get 0
                              i32.const 208
                              i32.add
                              local.get 20
                              i32.const 5
                              i32.shl
                              i32.add
                              local.tee 1
                              local.get 2
                              i64.load align=1
                              i64.store align=1
                              local.get 1
                              i32.const 8
                              i32.add
                              local.get 25
                              i64.store align=1
                              local.get 1
                              i32.const 16
                              i32.add
                              local.get 27
                              i64.store align=1
                              local.get 1
                              i32.const 24
                              i32.add
                              local.get 26
                              i64.store align=1
                              local.get 20
                              i32.const 1
                              i32.add
                              local.tee 20
                              i32.const 12
                              i32.ne
                              br_if 0 (;@13;)
                            end
                            br 3 (;@9;)
                          end
                          br 5 (;@6;)
                        end
                        i32.const 1049456
                        i32.const 32
                        call 14
                        i32.const 1057247
                        i32.const 1
                        call 14
                        i32.const 0
                        call 5
                        i32.const 1
                        call 25
                        unreachable
                      end
                      br 5 (;@4;)
                    end
                    block  ;; label = @9
                      local.get 12
                      i32.load offset=164
                      local.tee 3
                      local.get 12
                      i32.load offset=160
                      local.tee 4
                      i32.ge_u
                      if  ;; label = @10
                        local.get 3
                        local.get 14
                        i32.gt_u
                        br_if 8 (;@2;)
                        local.get 0
                        i32.const 808
                        i32.add
                        i64.const 0
                        i64.store
                        local.get 0
                        i32.const 800
                        i32.add
                        i64.const 0
                        i64.store
                        local.get 0
                        i32.const 792
                        i32.add
                        i64.const 0
                        i64.store
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 4
                        local.get 8
                        i32.add
                        local.set 2
                        block  ;; label = @11
                          local.get 3
                          local.get 4
                          i32.sub
                          local.tee 1
                          i32.const 66
                          i32.sub
                          i32.const 2
                          i32.ge_u
                          if  ;; label = @12
                            local.get 1
                            i32.const 32
                            i32.eq
                            if  ;; label = @13
                              local.get 2
                              i32.const 8
                              i32.add
                              i64.load align=1
                              local.set 26
                              local.get 2
                              i32.const 16
                              i32.add
                              i64.load align=1
                              local.set 27
                              local.get 2
                              i64.load align=1
                              local.set 25
                              local.get 0
                              i32.const 624
                              i32.add
                              local.tee 1
                              i32.const 24
                              i32.add
                              local.get 2
                              i32.const 24
                              i32.add
                              i64.load align=1
                              i64.store
                              local.get 1
                              i32.const 16
                              i32.add
                              local.get 27
                              i64.store
                              local.get 1
                              i32.const 8
                              i32.add
                              local.get 26
                              i64.store
                              local.get 0
                              local.get 25
                              i64.store offset=624
                              br 2 (;@11;)
                            end
                            br 5 (;@7;)
                          end
                          local.get 2
                          i32.const 3
                          i32.add
                          local.set 3
                          i32.const 0
                          local.set 4
                          loop  ;; label = @12
                            local.get 3
                            i32.const 1
                            i32.sub
                            i32.load8_u
                            local.tee 6
                            i32.const 9
                            i32.add
                            local.set 2
                            local.get 0
                            i32.const 784
                            i32.add
                            local.tee 7
                            local.get 4
                            i32.add
                            local.get 3
                            i32.load8_u
                            local.tee 5
                            i32.const 48
                            i32.sub
                            local.tee 1
                            i32.const 0
                            local.get 1
                            i32.const 255
                            i32.and
                            i32.const 10
                            i32.lt_u
                            select
                            local.get 5
                            i32.const 87
                            i32.sub
                            i32.const 0
                            local.get 5
                            i32.const 97
                            i32.sub
                            i32.const 255
                            i32.and
                            i32.const 6
                            i32.lt_u
                            select
                            i32.or
                            local.get 5
                            i32.const 55
                            i32.sub
                            i32.const 0
                            local.get 5
                            i32.const 65
                            i32.sub
                            i32.const 255
                            i32.and
                            i32.const 6
                            i32.lt_u
                            select
                            i32.or
                            local.get 6
                            i32.const 0
                            local.get 6
                            i32.const 48
                            i32.sub
                            i32.const 255
                            i32.and
                            i32.const 10
                            i32.lt_u
                            select
                            local.get 2
                            i32.const 0
                            local.get 6
                            i32.const 97
                            i32.sub
                            i32.const 255
                            i32.and
                            i32.const 6
                            i32.lt_u
                            select
                            i32.or
                            local.get 2
                            i32.const 0
                            local.get 6
                            i32.const 65
                            i32.sub
                            i32.const 255
                            i32.and
                            i32.const 6
                            i32.lt_u
                            select
                            i32.or
                            i32.const 4
                            i32.shl
                            i32.or
                            i32.store8
                            local.get 3
                            i32.const 2
                            i32.add
                            local.set 3
                            local.get 4
                            i32.const 1
                            i32.add
                            local.tee 4
                            i32.const 32
                            i32.ne
                            br_if 0 (;@12;)
                          end
                          local.get 0
                          i32.const 624
                          i32.add
                          local.tee 1
                          i32.const 24
                          i32.add
                          local.get 7
                          i32.const 24
                          i32.add
                          i64.load
                          i64.store
                          local.get 1
                          i32.const 16
                          i32.add
                          local.get 7
                          i32.const 16
                          i32.add
                          i64.load
                          i64.store
                          local.get 1
                          i32.const 8
                          i32.add
                          local.get 7
                          i32.const 8
                          i32.add
                          i64.load
                          i64.store
                          local.get 0
                          local.get 0
                          i64.load offset=784
                          i64.store offset=624
                        end
                        local.get 0
                        i32.const 799
                        i32.add
                        local.tee 19
                        local.get 0
                        i32.const 8
                        i32.add
                        local.tee 21
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 807
                        i32.add
                        local.tee 4
                        local.get 0
                        i32.const 16
                        i32.add
                        local.tee 10
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 815
                        i32.add
                        local.tee 13
                        local.get 0
                        i32.const 24
                        i32.add
                        local.tee 11
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 831
                        i32.add
                        local.get 0
                        i32.const 80
                        i32.add
                        local.tee 1
                        i32.const 8
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 839
                        i32.add
                        local.get 1
                        i32.const 16
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 847
                        i32.add
                        local.tee 6
                        local.get 1
                        i32.const 24
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 1048601
                        i32.load align=1
                        i32.store offset=787 align=1
                        local.get 0
                        i32.const 1048598
                        i32.load align=1
                        i32.store offset=784
                        local.get 0
                        local.get 0
                        i64.load
                        i64.store offset=791 align=1
                        local.get 0
                        local.get 0
                        i64.load offset=80
                        i64.store offset=823 align=1
                        local.get 0
                        i32.const 879
                        i32.add
                        local.tee 5
                        local.get 0
                        i32.const 112
                        i32.add
                        local.tee 1
                        i32.const 24
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 871
                        i32.add
                        local.get 1
                        i32.const 16
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 863
                        i32.add
                        local.tee 7
                        local.get 1
                        i32.const 8
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        local.get 0
                        i64.load offset=112
                        i64.store offset=855 align=1
                        local.get 0
                        i32.const 1280
                        i32.add
                        local.tee 15
                        local.get 0
                        i32.const 784
                        i32.add
                        local.tee 18
                        i32.const 103
                        call 33
                        local.get 0
                        i32.const 1328
                        i32.add
                        local.tee 17
                        i32.const 24
                        i32.add
                        local.tee 2
                        i64.const 0
                        i64.store
                        local.get 17
                        i32.const 16
                        i32.add
                        local.tee 1
                        i64.const 0
                        i64.store
                        local.get 17
                        i32.const 8
                        i32.add
                        local.tee 9
                        i64.const 0
                        i64.store
                        local.get 0
                        i64.const 0
                        i64.store offset=1328
                        local.get 15
                        local.get 17
                        i32.const 32
                        i32.const 1
                        call 6
                        local.get 0
                        i32.const 656
                        i32.add
                        local.tee 22
                        i32.const 24
                        i32.add
                        local.tee 3
                        local.get 2
                        i64.load
                        i64.store
                        local.get 22
                        i32.const 16
                        i32.add
                        local.tee 2
                        local.get 1
                        i64.load
                        i64.store
                        local.get 22
                        i32.const 8
                        i32.add
                        local.tee 1
                        local.get 9
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=1328
                        i64.store offset=656
                        local.get 15
                        call 7
                        local.get 0
                        i32.const 927
                        i32.add
                        i64.const 0
                        i64.store align=1
                        local.get 0
                        i32.const 919
                        i32.add
                        i64.const 0
                        i64.store align=1
                        local.get 0
                        i32.const 911
                        i32.add
                        i64.const 0
                        i64.store align=1
                        local.get 19
                        local.get 21
                        i64.load
                        i64.store align=1
                        local.get 4
                        local.get 10
                        i64.load
                        i64.store align=1
                        local.get 13
                        local.get 11
                        i64.load
                        i64.store align=1
                        local.get 0
                        i64.const 0
                        i64.store offset=903 align=1
                        local.get 0
                        i32.const 1048605
                        i32.load align=1
                        i32.store offset=784
                        local.get 0
                        i32.const 1048608
                        i32.load align=1
                        i32.store offset=787 align=1
                        local.get 0
                        i64.const 0
                        i64.store offset=831 align=1
                        local.get 0
                        local.get 0
                        i64.load
                        i64.store offset=791 align=1
                        local.get 0
                        local.get 28
                        i64.store offset=823 align=1
                        local.get 7
                        local.get 0
                        i32.const 48
                        i32.add
                        local.tee 7
                        i32.const 24
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        i32.const 855
                        i32.add
                        local.get 7
                        i32.const 16
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 6
                        local.get 7
                        i32.const 8
                        i32.add
                        i64.load
                        i64.store align=1
                        local.get 0
                        local.get 0
                        i64.load offset=48
                        i64.store offset=839 align=1
                        local.get 0
                        i32.const 895
                        i32.add
                        local.get 3
                        i64.load
                        i64.store align=1
                        local.get 18
                        i32.const 103
                        i32.add
                        local.get 2
                        i64.load
                        i64.store align=1
                        local.get 5
                        local.get 1
                        i64.load
                        i64.store align=1
                        local.get 0
                        local.get 0
                        i64.load offset=656
                        i64.store offset=871 align=1
                        local.get 0
                        i32.const 688
                        i32.add
                        local.tee 1
                        local.get 18
                        i32.const 151
                        call 33
                        local.get 1
                        local.get 0
                        i32.const 144
                        i32.add
                        i32.const 32
                        i32.const 1
                        call 8
                        local.get 0
                        i32.const 1
                        i32.store8 offset=696
                        local.get 0
                        i64.load8_u offset=687
                        local.set 34
                        local.get 0
                        i32.load8_u offset=686
                        local.set 1
                        local.get 18
                        i32.const 8
                        i32.add
                        local.tee 7
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 18
                        call 2
                        local.get 9
                        local.get 7
                        i64.load
                        local.tee 29
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=1328
                        local.get 1
                        i32.const 7
                        i32.shr_u
                        i64.extend_i32_u
                        local.get 1
                        i32.const 1
                        i32.and
                        i64.extend_i32_u
                        local.set 31
                        local.get 1
                        i32.const 6
                        i32.shr_u
                        i32.const 1
                        i32.and
                        i64.extend_i32_u
                        local.set 32
                        local.get 1
                        i32.const 5
                        i32.shr_u
                        i32.const 1
                        i32.and
                        i64.extend_i32_u
                        local.set 33
                        local.get 1
                        i32.const 4
                        i32.shr_u
                        i32.const 1
                        i32.and
                        i64.extend_i32_u
                        local.set 28
                        local.get 1
                        i32.const 3
                        i32.shr_u
                        i32.const 1
                        i32.and
                        i64.extend_i32_u
                        local.set 26
                        local.get 1
                        i32.const 2
                        i32.shr_u
                        i32.const 1
                        i32.and
                        i64.extend_i32_u
                        local.set 27
                        local.get 1
                        i32.const 1
                        i32.shr_u
                        i32.const 1
                        i32.and
                        i64.extend_i32_u
                        local.set 25
                        local.get 29
                        i32.wrap_i64
                        i32.const 255
                        i32.and
                        if  ;; label = @11
                          local.get 17
                          call 7
                          local.get 17
                          call 2
                          local.get 0
                          i32.const 0
                          i32.store8 offset=1336
                        end
                        i64.const 15
                        i64.shl
                        local.set 35
                        local.get 31
                        i64.const 8
                        i64.shl
                        local.set 29
                        local.get 32
                        i64.const 14
                        i64.shl
                        local.set 30
                        local.get 33
                        i64.const 13
                        i64.shl
                        local.get 28
                        i64.const 12
                        i64.shl
                        local.get 26
                        i64.const 11
                        i64.shl
                        local.get 27
                        i64.const 10
                        i64.shl
                        local.get 25
                        i64.const 9
                        i64.shl
                        local.get 0
                        i32.const 1328
                        i32.add
                        local.tee 2
                        local.get 0
                        i32.const 656
                        i32.add
                        i32.const 32
                        i32.const 1
                        call 9
                        local.get 0
                        i32.const 712
                        i32.add
                        local.get 9
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=1328
                        i64.store offset=704
                        local.get 7
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 0
                        i32.const 784
                        i32.add
                        local.tee 1
                        call 2
                        local.get 9
                        local.get 7
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=1328
                        local.get 2
                        call 2
                        local.get 2
                        i32.const 1
                        call 10
                        local.get 0
                        i32.const 728
                        i32.add
                        local.get 9
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=1328
                        i64.store offset=720
                        local.get 7
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 1
                        call 2
                        local.get 0
                        i32.const 744
                        i32.add
                        local.get 7
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=736
                        i32.const 0
                        local.set 3
                        loop  ;; label = @11
                          local.get 7
                          i32.const 0
                          i32.store8
                          local.get 0
                          i64.const 0
                          i64.store offset=784
                          local.get 0
                          i32.const 784
                          i32.add
                          call 2
                          local.get 9
                          local.get 7
                          i64.load
                          local.tee 25
                          i64.store
                          local.get 0
                          local.get 0
                          i64.load offset=784
                          i64.store offset=1328
                          local.get 25
                          i32.wrap_i64
                          i32.const 255
                          i32.and
                          if  ;; label = @12
                            local.get 0
                            i32.const 1328
                            i32.add
                            local.tee 1
                            call 7
                            local.get 1
                            call 2
                            local.get 0
                            i32.const 0
                            i32.store8 offset=1336
                          end
                          local.get 0
                          i32.const 1328
                          i32.add
                          local.get 0
                          i32.const 208
                          i32.add
                          local.get 3
                          i32.add
                          i32.const 32
                          i32.const 1
                          call 9
                          local.get 0
                          i32.const 1280
                          i32.add
                          local.tee 1
                          i32.const 8
                          i32.add
                          local.get 9
                          i64.load
                          i64.store
                          local.get 0
                          local.get 0
                          i64.load offset=1328
                          i64.store offset=1280
                          block  ;; label = @12
                            local.get 0
                            i32.load8_u offset=744
                            i32.eqz
                            if  ;; label = @13
                              local.get 0
                              i32.const 736
                              i32.add
                              local.get 0
                              i32.const 704
                              i32.add
                              local.get 1
                              call 11
                              br 1 (;@12;)
                            end
                            local.get 7
                            i32.const 0
                            i32.store8
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            local.get 0
                            i32.const 784
                            i32.add
                            call 2
                            local.get 9
                            local.get 7
                            i64.load
                            i64.store
                            local.get 0
                            local.get 0
                            i64.load offset=784
                            i64.store offset=1328
                            local.get 0
                            i32.const 1328
                            i32.add
                            local.tee 1
                            local.get 0
                            i32.const 704
                            i32.add
                            local.get 0
                            i32.const 1280
                            i32.add
                            call 11
                            local.get 0
                            i32.const 0
                            i32.store8 offset=744
                            local.get 0
                            i64.load offset=1328
                            local.set 25
                            local.get 0
                            local.get 0
                            i64.load offset=736
                            i64.store offset=1328
                            local.get 0
                            local.get 25
                            i64.store offset=736
                            local.get 1
                            call 7
                          end
                          local.get 0
                          i32.const 704
                          i32.add
                          local.get 0
                          i32.const 736
                          i32.add
                          local.tee 2
                          local.get 0
                          i32.const 1280
                          i32.add
                          local.tee 1
                          call 12
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1288
                          local.get 0
                          i32.const 1
                          i32.store8 offset=744
                          local.get 0
                          i32.const 1
                          i32.store8 offset=712
                          local.get 0
                          i32.const 720
                          i32.add
                          local.get 2
                          call 34
                          local.get 1
                          call 7
                          local.get 3
                          i32.const 32
                          i32.add
                          local.tee 3
                          i32.const 384
                          i32.ne
                          br_if 0 (;@11;)
                        end
                        i32.const 0
                        local.set 16
                        local.get 0
                        i32.const 784
                        i32.add
                        local.tee 1
                        i32.const 8
                        i32.add
                        local.tee 3
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 1
                        call 2
                        local.get 0
                        i32.const 1328
                        i32.add
                        local.tee 2
                        i32.const 8
                        i32.add
                        local.tee 1
                        local.get 3
                        i64.load
                        local.tee 27
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=1328
                        local.get 29
                        local.get 34
                        i64.or
                        i64.or
                        i64.or
                        i64.or
                        i64.or
                        i64.or
                        local.get 27
                        i32.wrap_i64
                        i32.const 255
                        i32.and
                        if  ;; label = @11
                          local.get 2
                          call 7
                          local.get 2
                          call 2
                          local.get 0
                          i32.const 0
                          i32.store8 offset=1336
                        end
                        local.get 0
                        i32.const 1328
                        i32.add
                        local.get 0
                        i32.const 624
                        i32.add
                        i32.const 32
                        i32.const 1
                        call 9
                        local.get 0
                        i32.const 752
                        i32.add
                        local.tee 2
                        i32.const 8
                        i32.add
                        local.get 1
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=1328
                        i64.store offset=752
                        local.get 0
                        i32.const 720
                        i32.add
                        local.tee 1
                        local.get 2
                        call 34
                        local.get 1
                        i64.const 1
                        call 4
                        local.get 0
                        i32.const 1
                        i32.store8 offset=728
                        local.get 3
                        i32.const 1048594
                        i32.load align=1
                        i32.store
                        local.get 0
                        i32.const 1048586
                        i64.load align=1
                        i64.store offset=784
                        local.get 0
                        i32.const 796
                        i32.add
                        local.get 0
                        i32.const 208
                        i32.add
                        i32.const 384
                        memory.copy
                        local.get 0
                        i32.const 768
                        i32.add
                        local.get 0
                        i32.const 784
                        i32.add
                        local.tee 2
                        i32.const 396
                        call 33
                        local.get 3
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 2
                        call 2
                        local.get 0
                        i32.const 1192
                        i32.add
                        local.get 3
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=1184
                        local.get 3
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 2
                        call 2
                        local.get 0
                        i32.const 1208
                        i32.add
                        local.get 3
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=1200
                        local.get 3
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 2
                        call 2
                        local.get 0
                        i32.const 1224
                        i32.add
                        local.get 3
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=1216
                        local.get 3
                        i32.const 0
                        i32.store8
                        local.get 0
                        i64.const 0
                        i64.store offset=784
                        local.get 2
                        call 2
                        local.get 0
                        i32.const 1240
                        i32.add
                        local.get 3
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=784
                        i64.store offset=1232
                        local.get 0
                        i32.const 1248
                        i32.add
                        local.tee 1
                        i32.const 24
                        i32.add
                        local.tee 21
                        i64.const 0
                        i64.store
                        local.get 1
                        i32.const 16
                        i32.add
                        local.tee 10
                        i64.const 0
                        i64.store
                        local.get 1
                        i32.const 8
                        i32.add
                        i64.const 0
                        i64.store
                        local.get 0
                        i64.const 0
                        i64.store offset=1248
                        local.get 0
                        i32.const 1280
                        i32.add
                        local.tee 1
                        i32.const 24
                        i32.add
                        local.tee 13
                        i64.const 0
                        i64.store
                        local.get 1
                        i32.const 16
                        i32.add
                        local.tee 11
                        i64.const 0
                        i64.store
                        local.get 1
                        i32.const 8
                        i32.add
                        i64.const 0
                        i64.store
                        local.get 0
                        i64.const 0
                        i64.store offset=1280
                        local.get 8
                        i32.const 3
                        i32.add
                        local.set 6
                        local.get 30
                        i64.or
                        local.get 35
                        i64.or
                        local.set 28
                        local.get 0
                        i32.const 827
                        i32.add
                        local.set 17
                        local.get 0
                        i32.const 795
                        i32.add
                        local.set 18
                        i32.const 21
                        local.set 22
                        local.get 2
                        i32.const 8
                        i32.add
                        local.set 9
                        loop  ;; label = @11
                          local.get 12
                          local.get 22
                          i32.const 3
                          i32.shl
                          i32.add
                          local.tee 1
                          i32.load offset=4
                          local.tee 3
                          local.get 1
                          i32.load
                          local.tee 4
                          i32.lt_u
                          br_if 8 (;@3;)
                          local.get 3
                          local.get 14
                          i32.gt_u
                          br_if 2 (;@9;)
                          local.get 0
                          i32.const 784
                          i32.add
                          local.tee 1
                          i32.const 24
                          i32.add
                          local.tee 7
                          i64.const 0
                          i64.store
                          local.get 1
                          i32.const 16
                          i32.add
                          local.tee 2
                          i64.const 0
                          i64.store
                          local.get 9
                          i64.const 0
                          i64.store
                          local.get 0
                          i64.const 0
                          i64.store offset=784
                          block  ;; label = @12
                            local.get 3
                            local.get 4
                            i32.sub
                            local.tee 1
                            i32.const 66
                            i32.sub
                            i32.const 2
                            i32.ge_u
                            if  ;; label = @13
                              local.get 1
                              i32.const 32
                              i32.eq
                              if  ;; label = @14
                                local.get 4
                                local.get 8
                                i32.add
                                local.tee 2
                                i32.const 8
                                i32.add
                                i64.load align=1
                                local.set 26
                                local.get 2
                                i32.const 16
                                i32.add
                                i64.load align=1
                                local.set 27
                                local.get 2
                                i64.load align=1
                                local.set 25
                                local.get 0
                                i32.const 1328
                                i32.add
                                local.tee 1
                                i32.const 24
                                i32.add
                                local.get 2
                                i32.const 24
                                i32.add
                                i64.load align=1
                                i64.store
                                local.get 1
                                i32.const 16
                                i32.add
                                local.get 27
                                i64.store
                                local.get 1
                                i32.const 8
                                i32.add
                                local.get 26
                                i64.store
                                local.get 0
                                local.get 25
                                i64.store offset=1328
                                br 2 (;@12;)
                              end
                              local.get 0
                              i32.const 1328
                              i32.add
                              local.tee 2
                              i32.const 8
                              i32.add
                              local.tee 1
                              i32.const 0
                              i32.store8
                              local.get 0
                              i64.const 0
                              i64.store offset=1328
                              local.get 2
                              call 2
                              local.get 0
                              i32.const 1360
                              i32.add
                              local.tee 2
                              i32.const 8
                              i32.add
                              local.get 1
                              i64.load
                              i64.store
                              local.get 0
                              local.get 0
                              i64.load offset=1328
                              i64.store offset=1360
                              br 12 (;@1;)
                            end
                            local.get 4
                            local.get 6
                            i32.add
                            local.set 3
                            i32.const 0
                            local.set 4
                            loop  ;; label = @13
                              local.get 3
                              i32.const 1
                              i32.sub
                              i32.load8_u
                              local.tee 15
                              i32.const 9
                              i32.add
                              local.set 5
                              local.get 0
                              i32.const 784
                              i32.add
                              local.get 4
                              i32.add
                              local.get 3
                              i32.load8_u
                              local.tee 19
                              i32.const 48
                              i32.sub
                              local.tee 1
                              i32.const 0
                              local.get 1
                              i32.const 255
                              i32.and
                              i32.const 10
                              i32.lt_u
                              select
                              local.get 19
                              i32.const 87
                              i32.sub
                              i32.const 0
                              local.get 19
                              i32.const 97
                              i32.sub
                              i32.const 255
                              i32.and
                              i32.const 6
                              i32.lt_u
                              select
                              i32.or
                              local.get 19
                              i32.const 55
                              i32.sub
                              i32.const 0
                              local.get 19
                              i32.const 65
                              i32.sub
                              i32.const 255
                              i32.and
                              i32.const 6
                              i32.lt_u
                              select
                              i32.or
                              local.get 15
                              i32.const 0
                              local.get 15
                              i32.const 48
                              i32.sub
                              i32.const 255
                              i32.and
                              i32.const 10
                              i32.lt_u
                              select
                              local.get 5
                              i32.const 0
                              local.get 15
                              i32.const 97
                              i32.sub
                              i32.const 255
                              i32.and
                              i32.const 6
                              i32.lt_u
                              select
                              i32.or
                              local.get 5
                              i32.const 0
                              local.get 15
                              i32.const 65
                              i32.sub
                              i32.const 255
                              i32.and
                              i32.const 6
                              i32.lt_u
                              select
                              i32.or
                              i32.const 4
                              i32.shl
                              i32.or
                              i32.store8
                              local.get 3
                              i32.const 2
                              i32.add
                              local.set 3
                              local.get 4
                              i32.const 1
                              i32.add
                              local.tee 4
                              i32.const 32
                              i32.ne
                              br_if 0 (;@13;)
                            end
                            local.get 0
                            i32.const 1328
                            i32.add
                            local.tee 1
                            i32.const 24
                            i32.add
                            local.get 7
                            i64.load
                            i64.store
                            local.get 1
                            i32.const 16
                            i32.add
                            local.get 2
                            i64.load
                            i64.store
                            local.get 1
                            i32.const 8
                            i32.add
                            local.get 9
                            i64.load
                            i64.store
                            local.get 0
                            local.get 0
                            i64.load offset=784
                            i64.store offset=1328
                          end
                          local.get 9
                          i32.const 0
                          i32.store8
                          local.get 0
                          i64.const 0
                          i64.store offset=784
                          local.get 0
                          i32.const 784
                          i32.add
                          call 2
                          local.get 0
                          i32.const 1360
                          i32.add
                          local.tee 1
                          i32.const 8
                          i32.add
                          local.tee 15
                          local.get 9
                          i64.load
                          local.tee 25
                          i64.store
                          local.get 0
                          local.get 0
                          i64.load offset=784
                          i64.store offset=1360
                          local.get 25
                          i32.wrap_i64
                          i32.const 255
                          i32.and
                          if  ;; label = @12
                            local.get 1
                            call 7
                            local.get 1
                            call 2
                            local.get 0
                            i32.const 0
                            i32.store8 offset=1368
                          end
                          local.get 0
                          i32.const 1360
                          i32.add
                          local.get 0
                          i32.const 1328
                          i32.add
                          i32.const 32
                          i32.const 1
                          call 9
                          local.get 0
                          i32.const 1320
                          i32.add
                          local.get 15
                          i64.load
                          i64.store
                          local.get 0
                          local.get 0
                          i64.load offset=1360
                          i64.store offset=1312
                          local.get 28
                          i32.wrap_i64
                          i32.const 1
                          i32.and
                          local.set 2
                          local.get 0
                          i32.load8_u offset=1240
                          if  ;; label = @12
                            local.get 0
                            i32.const 1232
                            i32.add
                            local.tee 1
                            call 7
                            local.get 1
                            call 2
                            local.get 0
                            i32.const 0
                            i32.store8 offset=1240
                          end
                          local.get 0
                          i32.const 1232
                          i32.add
                          local.get 2
                          call 10
                          block  ;; label = @12
                            local.get 0
                            i32.load8_u offset=1224
                            i32.eqz
                            if  ;; label = @13
                              local.get 0
                              i32.const 1216
                              i32.add
                              local.get 0
                              i32.const 1312
                              i32.add
                              local.get 0
                              i32.const 768
                              i32.add
                              call 11
                              br 1 (;@12;)
                            end
                            local.get 9
                            i32.const 0
                            i32.store8
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            local.get 0
                            i32.const 784
                            i32.add
                            call 2
                            local.get 15
                            local.get 9
                            i64.load
                            i64.store
                            local.get 0
                            local.get 0
                            i64.load offset=784
                            i64.store offset=1360
                            local.get 0
                            i32.const 1360
                            i32.add
                            local.tee 1
                            local.get 0
                            i32.const 1312
                            i32.add
                            local.get 0
                            i32.const 768
                            i32.add
                            call 11
                            local.get 0
                            i32.const 0
                            i32.store8 offset=1224
                            local.get 0
                            i64.load offset=1360
                            local.set 25
                            local.get 0
                            local.get 0
                            i64.load offset=1216
                            i64.store offset=1360
                            local.get 0
                            local.get 25
                            i64.store offset=1216
                            local.get 1
                            call 7
                          end
                          local.get 0
                          i32.const 1312
                          i32.add
                          local.get 0
                          i32.const 1216
                          i32.add
                          local.tee 2
                          local.get 0
                          i32.const 768
                          i32.add
                          local.tee 1
                          call 12
                          local.get 0
                          i32.const 1
                          i32.store8 offset=776
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1224
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1320
                          local.get 2
                          local.get 0
                          i32.const 1232
                          i32.add
                          call 34
                          block  ;; label = @12
                            local.get 0
                            i32.load8_u offset=1192
                            i32.eqz
                            if  ;; label = @13
                              local.get 0
                              i32.const 1184
                              i32.add
                              local.get 1
                              local.get 2
                              call 13
                              br 1 (;@12;)
                            end
                            local.get 9
                            i32.const 0
                            i32.store8
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            local.get 0
                            i32.const 784
                            i32.add
                            call 2
                            local.get 15
                            local.get 9
                            i64.load
                            i64.store
                            local.get 0
                            local.get 0
                            i64.load offset=784
                            i64.store offset=1360
                            local.get 0
                            i32.const 1360
                            i32.add
                            local.tee 1
                            local.get 0
                            i32.const 768
                            i32.add
                            local.get 0
                            i32.const 1216
                            i32.add
                            call 13
                            local.get 0
                            i32.const 0
                            i32.store8 offset=1192
                            local.get 0
                            i64.load offset=1360
                            local.set 25
                            local.get 0
                            local.get 0
                            i64.load offset=1184
                            i64.store offset=1360
                            local.get 0
                            local.get 25
                            i64.store offset=1184
                            local.get 1
                            call 7
                          end
                          local.get 0
                          i32.const 1184
                          i32.add
                          local.get 0
                          i32.const 768
                          i32.add
                          local.get 0
                          i32.const 1216
                          i32.add
                          local.tee 1
                          call 12
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1224
                          local.get 0
                          i32.const 1
                          i32.store8 offset=776
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1192
                          block  ;; label = @12
                            local.get 0
                            i32.load8_u offset=1208
                            i32.eqz
                            if  ;; label = @13
                              local.get 0
                              i32.const 1200
                              i32.add
                              local.get 0
                              i32.const 1312
                              i32.add
                              local.get 1
                              call 11
                              br 1 (;@12;)
                            end
                            local.get 9
                            i32.const 0
                            i32.store8
                            local.get 0
                            i64.const 0
                            i64.store offset=784
                            local.get 0
                            i32.const 784
                            i32.add
                            call 2
                            local.get 15
                            local.get 9
                            i64.load
                            i64.store
                            local.get 0
                            local.get 0
                            i64.load offset=784
                            i64.store offset=1360
                            local.get 0
                            i32.const 1360
                            i32.add
                            local.tee 1
                            local.get 0
                            i32.const 1312
                            i32.add
                            local.get 0
                            i32.const 1216
                            i32.add
                            call 11
                            local.get 0
                            i32.const 0
                            i32.store8 offset=1208
                            local.get 0
                            i64.load offset=1360
                            local.set 25
                            local.get 0
                            local.get 0
                            i64.load offset=1200
                            i64.store offset=1360
                            local.get 0
                            local.get 25
                            i64.store offset=1200
                            local.get 1
                            call 7
                          end
                          local.get 22
                          i32.const 1
                          i32.add
                          local.set 22
                          local.get 0
                          i32.const 1312
                          i32.add
                          local.tee 5
                          local.get 0
                          i32.const 1200
                          i32.add
                          local.tee 4
                          local.get 0
                          i32.const 1216
                          i32.add
                          local.tee 7
                          call 12
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1224
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1208
                          local.get 0
                          i32.const 1
                          i32.store8 offset=1320
                          local.get 0
                          i32.const 1184
                          i32.add
                          local.tee 3
                          local.get 0
                          i32.const 1248
                          i32.add
                          local.tee 2
                          i32.const 32
                          i32.const 1
                          call 6
                          local.get 4
                          local.get 0
                          i32.const 1280
                          i32.add
                          local.tee 1
                          i32.const 32
                          i32.const 1
                          call 6
                          local.get 9
                          i32.const 1048584
                          i32.load16_u align=1
                          i32.store16
                          local.get 18
                          local.get 0
                          i64.load offset=1248
                          i64.store align=1
                          local.get 18
                          i32.const 8
                          i32.add
                          local.get 2
                          i32.const 8
                          i32.add
                          i64.load
                          i64.store align=1
                          local.get 18
                          i32.const 16
                          i32.add
                          local.get 10
                          i64.load
                          i64.store align=1
                          local.get 18
                          i32.const 24
                          i32.add
                          local.get 21
                          i64.load
                          i64.store align=1
                          local.get 17
                          local.get 0
                          i64.load offset=1280
                          i64.store align=1
                          local.get 17
                          i32.const 8
                          i32.add
                          local.get 1
                          i32.const 8
                          i32.add
                          i64.load
                          i64.store align=1
                          local.get 17
                          i32.const 16
                          i32.add
                          local.get 11
                          i64.load
                          i64.store align=1
                          local.get 17
                          i32.const 24
                          i32.add
                          local.get 13
                          i64.load
                          i64.store align=1
                          local.get 0
                          i32.const 1048576
                          i64.load align=1
                          i64.store offset=784
                          local.get 0
                          local.get 16
                          i32.store8 offset=794
                          local.get 0
                          i32.const 1360
                          i32.add
                          local.get 0
                          i32.const 784
                          i32.add
                          local.tee 19
                          i32.const 75
                          call 33
                          local.get 0
                          i32.const 768
                          i32.add
                          local.tee 1
                          call 7
                          local.get 1
                          i32.const 8
                          i32.add
                          local.tee 2
                          local.get 15
                          i64.load
                          i64.store
                          local.get 0
                          local.get 0
                          i64.load offset=1360
                          i64.store offset=768
                          local.get 28
                          i64.const 1
                          i64.shr_u
                          local.set 28
                          local.get 5
                          call 7
                          local.get 16
                          i32.const 1
                          i32.add
                          local.tee 16
                          i32.const 16
                          i32.ne
                          br_if 0 (;@11;)
                        end
                        local.get 19
                        i32.const 8
                        i32.add
                        local.tee 1
                        local.get 2
                        i64.load
                        i64.store
                        local.get 0
                        local.get 0
                        i64.load offset=768
                        i64.store offset=784
                        local.get 0
                        i32.const 1232
                        i32.add
                        call 7
                        local.get 7
                        call 7
                        local.get 4
                        call 7
                        local.get 3
                        call 7
                        local.get 19
                        local.get 0
                        i32.const 176
                        i32.add
                        i32.const 32
                        i32.const 1
                        call 8
                        local.get 1
                        i32.const 1
                        i32.store8
                        local.get 19
                        call 7
                        local.get 0
                        i32.const 752
                        i32.add
                        call 7
                        local.get 0
                        i32.const 736
                        i32.add
                        call 7
                        local.get 0
                        i32.const 720
                        i32.add
                        call 7
                        local.get 0
                        i32.const 704
                        i32.add
                        call 7
                        local.get 0
                        i32.const 688
                        i32.add
                        call 7
                        local.get 0
                        i32.const 32
                        i32.add
                        call 7
                        local.get 24
                        if  ;; label = @11
                          local.get 8
                          call 98
                        end
                        local.get 23
                        if  ;; label = @11
                          local.get 12
                          call 98
                        end
                        local.get 0
                        i32.const 1376
                        i32.add
                        global.set 0
                        return
                      end
                      br 6 (;@3;)
                    end
                    br 6 (;@2;)
                  end
                  local.get 0
                  i32.const 208
                  i32.add
                  local.tee 2
                  i32.const 8
                  i32.add
                  local.tee 1
                  i32.const 0
                  i32.store8
                  local.get 0
                  i64.const 0
                  i64.store offset=208
                  local.get 2
                  call 2
                  local.get 0
                  i32.const 1328
                  i32.add
                  local.tee 2
                  i32.const 8
                  i32.add
                  local.get 1
                  i64.load
                  i64.store
                  local.get 0
                  local.get 0
                  i64.load offset=208
                  i64.store offset=1328
                  br 6 (;@1;)
                end
                local.get 0
                i32.const 1328
                i32.add
                local.tee 2
                i32.const 8
                i32.add
                local.tee 1
                i32.const 0
                i32.store8
                local.get 0
                i64.const 0
                i64.store offset=1328
                local.get 2
                call 2
                local.get 0
                i32.const 1280
                i32.add
                local.tee 2
                i32.const 8
                i32.add
                local.get 1
                i64.load
                i64.store
                local.get 0
                local.get 0
                i64.load offset=1328
                i64.store offset=1280
                br 5 (;@1;)
              end
              local.get 3
              local.get 2
              call 31
              unreachable
            end
            local.get 2
            local.get 14
            i32.const 1049440
            call 32
            unreachable
          end
          local.get 0
          i32.const 784
          i32.add
          local.tee 2
          i32.const 8
          i32.add
          local.tee 1
          i32.const 0
          i32.store8
          local.get 0
          i64.const 0
          i64.store offset=784
          local.get 2
          call 2
          local.get 0
          i32.const 208
          i32.add
          local.tee 2
          i32.const 8
          i32.add
          local.get 1
          i64.load
          i64.store
          local.get 0
          local.get 0
          i64.load offset=784
          i64.store offset=208
          br 2 (;@1;)
        end
        local.get 4
        local.get 3
        call 31
        unreachable
      end
      local.get 3
      local.get 14
      i32.const 1049440
      call 32
      unreachable
    end
    local.get 2
    i64.const 0
    call 30
    local.get 2
    i64.const 1
    call 30
    i32.const 71
    call 25
    unreachable)
  (func (;27;) (type 4) (param i32 i32 i32)
    (local i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1048780
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    local.get 3
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=40
    local.get 3
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call 39
    unreachable)
  (func (;28;) (type 2) (param i32)
    (local i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 1
    global.set 0
    local.get 1
    i32.const 0
    i32.store offset=24
    local.get 1
    i32.const 1
    i32.store offset=12
    local.get 1
    i32.const 1048632
    i32.store offset=8
    local.get 1
    i64.const 4
    i64.store offset=16 align=4
    local.get 1
    i32.const 8
    i32.add
    local.get 0
    call 39
    unreachable)
  (func (;29;) (type 2) (param i32)
    (local i32 i32 i32 i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 1
    global.set 0
    local.get 1
    i32.const 2
    i32.store offset=12
    local.get 1
    i32.const 1057052
    i32.store offset=8
    local.get 1
    i64.const 1
    i64.store offset=20 align=4
    local.get 1
    local.get 1
    i32.const 40
    i32.add
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=32
    local.get 1
    local.get 0
    i32.store offset=40
    local.get 1
    local.get 1
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 1
    local.get 1
    i32.const 47
    i32.add
    local.get 1
    i32.const 8
    i32.add
    call 62
    local.get 1
    i32.load offset=4
    local.set 0
    local.get 1
    i32.load8_u
    local.tee 2
    i32.const 3
    i32.ne
    local.get 2
    i32.const 4
    i32.le_u
    i32.and
    i32.eqz
    if  ;; label = @1
      local.get 0
      i32.load
      local.set 2
      local.get 0
      i32.const 4
      i32.add
      i32.load
      local.tee 3
      i32.load
      local.tee 4
      if  ;; label = @2
        local.get 2
        local.get 4
        call_indirect (type 2)
      end
      local.get 3
      i32.load offset=4
      if  ;; label = @2
        local.get 2
        call 98
      end
      local.get 0
      call 98
    end
    local.get 1
    i32.const 48
    i32.add
    global.set 0
    unreachable)
  (func (;30;) (type 7) (param i32 i64)
    local.get 0
    local.get 1
    call 4
    local.get 0
    i32.const 1
    i32.store8 offset=8)
  (func (;31;) (type 3) (param i32 i32)
    (local i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    local.get 1
    i32.store offset=4
    local.get 2
    local.get 0
    i32.store
    local.get 2
    i32.const 2
    i32.store offset=12
    local.get 2
    i32.const 1049336
    i32.store offset=8
    local.get 2
    i64.const 2
    i64.store offset=20 align=4
    local.get 2
    local.get 2
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=40
    local.get 2
    local.get 2
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=32
    local.get 2
    local.get 2
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 2
    i32.const 8
    i32.add
    i32.const 1049440
    call 39
    unreachable)
  (func (;32;) (type 4) (param i32 i32 i32)
    (local i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1049284
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=40
    local.get 3
    local.get 3
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call 39
    unreachable)
  (func (;33;) (type 4) (param i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get 0
    i32.const 144
    i32.sub
    local.tee 3
    global.set 0
    i32.const 1057764
    i32.load8_u
    drop
    i32.const 2048
    call 97
    local.tee 12
    if  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        loop  ;; label = @11
                          i32.const 1057764
                          i32.load8_u
                          drop
                          local.get 10
                          i32.const 3
                          i32.shl
                          i32.const 1054548
                          i32.add
                          i32.load
                          local.set 7
                          block  ;; label = @12
                            i32.const 67
                            call 97
                            local.tee 9
                            if  ;; label = @13
                              local.get 9
                              local.get 7
                              i32.const 66
                              memory.copy
                              local.get 7
                              i32.const 3
                              i32.add
                              i32.const -4
                              i32.and
                              local.get 7
                              i32.sub
                              local.tee 6
                              i32.eqz
                              if  ;; label = @14
                                i32.const 0
                                local.set 6
                                br 2 (;@12;)
                              end
                              i32.const 0
                              local.set 4
                              loop  ;; label = @14
                                local.get 4
                                local.get 7
                                i32.add
                                i32.load8_u
                                i32.eqz
                                br_if 5 (;@9;)
                                local.get 6
                                local.get 4
                                i32.const 1
                                i32.add
                                local.tee 4
                                i32.ne
                                br_if 0 (;@14;)
                              end
                              br 1 (;@12;)
                            end
                            i32.const 67
                            call 29
                            unreachable
                          end
                          block  ;; label = @12
                            block  ;; label = @13
                              i32.const 16843008
                              local.get 6
                              local.get 7
                              i32.add
                              local.tee 5
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.const 16843008
                              local.get 5
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 5
                              i32.sub
                              local.get 5
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              if  ;; label = @14
                                local.get 6
                                local.set 5
                                br 1 (;@13;)
                              end
                              i32.const 16843008
                              local.get 7
                              local.get 6
                              i32.const 8
                              i32.or
                              local.tee 5
                              i32.add
                              local.tee 4
                              i32.load
                              local.tee 8
                              i32.sub
                              local.get 8
                              i32.or
                              i32.const 16843008
                              local.get 4
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              br_if 0 (;@13;)
                              i32.const 16843008
                              local.get 7
                              local.get 6
                              i32.const 16
                              i32.or
                              local.tee 5
                              i32.add
                              local.tee 4
                              i32.load
                              local.tee 8
                              i32.sub
                              local.get 8
                              i32.or
                              i32.const 16843008
                              local.get 4
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              br_if 0 (;@13;)
                              i32.const 16843008
                              local.get 7
                              local.get 6
                              i32.const 24
                              i32.or
                              local.tee 5
                              i32.add
                              local.tee 4
                              i32.load
                              local.tee 8
                              i32.sub
                              local.get 8
                              i32.or
                              i32.const 16843008
                              local.get 4
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              br_if 0 (;@13;)
                              i32.const 16843008
                              local.get 7
                              local.get 6
                              i32.const 32
                              i32.or
                              local.tee 5
                              i32.add
                              local.tee 4
                              i32.load
                              local.tee 8
                              i32.sub
                              local.get 8
                              i32.or
                              i32.const 16843008
                              local.get 4
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              br_if 0 (;@13;)
                              i32.const 16843008
                              local.get 7
                              local.get 6
                              i32.const 40
                              i32.or
                              local.tee 5
                              i32.add
                              local.tee 4
                              i32.load
                              local.tee 8
                              i32.sub
                              local.get 8
                              i32.or
                              i32.const 16843008
                              local.get 4
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              br_if 0 (;@13;)
                              i32.const 16843008
                              local.get 7
                              local.get 6
                              i32.const 48
                              i32.or
                              local.tee 5
                              i32.add
                              local.tee 4
                              i32.load
                              local.tee 8
                              i32.sub
                              local.get 8
                              i32.or
                              i32.const 16843008
                              local.get 4
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              br_if 0 (;@13;)
                              local.get 6
                              i32.const 56
                              i32.or
                              local.set 5
                              local.get 6
                              i32.const 3
                              i32.eq
                              br_if 0 (;@13;)
                              i32.const 16843008
                              local.get 5
                              local.get 7
                              i32.add
                              local.tee 4
                              i32.load
                              local.tee 8
                              i32.sub
                              local.get 8
                              i32.or
                              i32.const 16843008
                              local.get 4
                              i32.const 4
                              i32.add
                              i32.load
                              local.tee 4
                              i32.sub
                              local.get 4
                              i32.or
                              i32.and
                              i32.const -2139062144
                              i32.and
                              i32.const -2139062144
                              i32.ne
                              br_if 0 (;@13;)
                              local.get 6
                              i32.const 64
                              i32.or
                              local.tee 5
                              i32.const 66
                              i32.eq
                              br_if 1 (;@12;)
                            end
                            i32.const 66
                            local.get 5
                            i32.sub
                            local.set 6
                            local.get 5
                            local.get 7
                            i32.add
                            local.set 7
                            i32.const 0
                            local.set 4
                            loop  ;; label = @13
                              local.get 4
                              local.get 7
                              i32.add
                              i32.load8_u
                              i32.eqz
                              br_if 3 (;@10;)
                              local.get 6
                              local.get 4
                              i32.const 1
                              i32.add
                              local.tee 4
                              i32.ne
                              br_if 0 (;@13;)
                            end
                          end
                          local.get 9
                          i32.const 0
                          i32.store8 offset=66
                          local.get 3
                          i32.const 8
                          i32.add
                          local.tee 6
                          i32.const 8
                          i32.add
                          local.tee 5
                          i32.const 0
                          i32.store8
                          local.get 3
                          i64.const 0
                          i64.store offset=8
                          local.get 6
                          call 2
                          local.get 3
                          i32.const 96
                          i32.add
                          local.tee 8
                          i32.const 8
                          i32.add
                          local.tee 11
                          local.get 5
                          i64.load
                          i64.store
                          local.get 3
                          local.get 3
                          i64.load offset=8
                          i64.store offset=96
                          local.get 8
                          call 2
                          local.get 8
                          local.get 9
                          i32.const 0
                          call 17
                          local.get 5
                          local.get 11
                          i64.load
                          i64.store
                          local.get 3
                          local.get 3
                          i64.load offset=96
                          i64.store offset=8
                          local.get 9
                          i32.const 0
                          i32.store8
                          local.get 9
                          call 98
                          local.get 12
                          local.get 10
                          i32.const 4
                          i32.shl
                          i32.add
                          local.tee 4
                          i32.const 8
                          i32.add
                          local.get 5
                          i64.load
                          i64.store
                          local.get 4
                          local.get 3
                          i64.load offset=8
                          i64.store
                          local.get 10
                          i32.const 1
                          i32.add
                          local.tee 10
                          i32.const 128
                          i32.ne
                          br_if 0 (;@11;)
                        end
                        local.get 5
                        i32.const 0
                        i32.store8
                        local.get 3
                        i64.const 0
                        i64.store offset=8
                        local.get 6
                        call 2
                        local.get 3
                        i32.const 128
                        i32.add
                        local.tee 4
                        i32.const 8
                        i32.add
                        local.tee 7
                        local.get 5
                        i64.load
                        i64.store
                        local.get 3
                        local.get 3
                        i64.load offset=8
                        i64.store offset=128
                        local.get 4
                        call 2
                        local.get 4
                        i32.const 0
                        call 10
                        local.get 11
                        local.get 7
                        i64.load
                        i64.store
                        local.get 3
                        local.get 3
                        i64.load offset=128
                        i64.store offset=96
                        local.get 5
                        i32.const 0
                        i32.store8
                        local.get 3
                        i64.const 0
                        i64.store offset=8
                        local.get 6
                        call 2
                        local.get 7
                        local.get 5
                        i64.load
                        i64.store
                        local.get 3
                        local.get 3
                        i64.load offset=8
                        i64.store offset=128
                        local.get 4
                        call 2
                        local.get 4
                        i32.const 0
                        call 10
                        local.get 8
                        i32.const 24
                        i32.add
                        local.tee 9
                        local.get 7
                        i64.load
                        i64.store
                        local.get 3
                        local.get 3
                        i64.load offset=128
                        i64.store offset=112
                        i32.const 1057764
                        i32.load8_u
                        drop
                        i32.const 31
                        i32.const 1
                        call 99
                        local.tee 10
                        i32.eqz
                        br_if 4 (;@6;)
                        local.get 7
                        i32.const 0
                        i32.store8
                        local.get 3
                        i64.const 0
                        i64.store offset=128
                        local.get 4
                        call 2
                        local.get 3
                        i32.const 48
                        i32.add
                        local.get 7
                        i64.load
                        i64.store
                        local.get 5
                        local.get 11
                        i64.load
                        i64.store
                        local.get 6
                        i32.const 16
                        i32.add
                        local.get 8
                        i32.const 16
                        i32.add
                        i64.load
                        i64.store
                        local.get 6
                        i32.const 24
                        i32.add
                        local.get 9
                        i64.load
                        i64.store
                        local.get 3
                        local.get 3
                        i64.load offset=128
                        i64.store offset=40
                        local.get 3
                        i32.const 31
                        i32.store offset=80
                        local.get 3
                        local.get 10
                        i32.store offset=76
                        local.get 3
                        i32.const 31
                        i32.store offset=72
                        local.get 3
                        i32.const 2
                        i32.store offset=64
                        local.get 3
                        i64.const 240518168584
                        i64.store offset=56
                        local.get 3
                        local.get 3
                        i64.load offset=96
                        i64.store offset=8
                        local.get 3
                        i32.const 128
                        i32.store offset=92
                        local.get 3
                        local.get 12
                        i32.store offset=88
                        local.get 3
                        i32.const 128
                        i32.store offset=84
                        local.get 3
                        i32.const 0
                        i32.store offset=68
                        local.get 3
                        i32.const 40
                        i32.add
                        local.set 7
                        i32.const 0
                        local.set 5
                        local.get 2
                        local.set 6
                        loop  ;; label = @11
                          local.get 5
                          i32.const 31
                          i32.add
                          local.tee 4
                          local.get 2
                          i32.gt_u
                          br_if 6 (;@5;)
                          local.get 3
                          i32.load8_u offset=48
                          if  ;; label = @12
                            local.get 7
                            call 7
                            local.get 7
                            call 2
                            local.get 3
                            i32.const 0
                            i32.store8 offset=48
                          end
                          local.get 7
                          local.get 1
                          local.get 5
                          i32.add
                          i32.const 31
                          i32.const 1
                          call 9
                          local.get 3
                          i32.const 8
                          i32.add
                          local.tee 5
                          local.get 7
                          call 57
                          local.get 5
                          call 58
                          local.get 4
                          local.set 5
                          local.get 6
                          i32.const 31
                          i32.sub
                          local.tee 6
                          i32.const 30
                          i32.gt_u
                          br_if 0 (;@11;)
                        end
                        local.get 3
                        i32.load offset=68
                        local.set 5
                        local.get 2
                        local.get 4
                        i32.ne
                        if  ;; label = @11
                          local.get 1
                          local.get 4
                          i32.add
                          local.set 1
                          loop  ;; label = @12
                            local.get 3
                            i32.load offset=80
                            local.tee 2
                            local.get 5
                            i32.le_u
                            br_if 8 (;@4;)
                            local.get 3
                            i32.load offset=76
                            local.get 5
                            i32.add
                            local.get 1
                            i32.load8_u
                            i32.store8
                            local.get 3
                            local.get 3
                            i32.load offset=68
                            i32.const 1
                            i32.add
                            local.tee 5
                            i32.store offset=68
                            local.get 5
                            i32.const 30
                            i32.gt_u
                            if  ;; label = @13
                              local.get 3
                              i32.load offset=80
                              local.tee 2
                              i32.const 30
                              i32.le_u
                              br_if 10 (;@3;)
                              local.get 3
                              i32.load offset=76
                              local.set 2
                              local.get 3
                              i32.load8_u offset=48
                              if  ;; label = @14
                                local.get 7
                                call 7
                                local.get 7
                                call 2
                                local.get 3
                                i32.const 0
                                i32.store8 offset=48
                              end
                              local.get 7
                              local.get 2
                              i32.const 31
                              i32.const 1
                              call 9
                              local.get 3
                              i32.const 8
                              i32.add
                              local.tee 2
                              local.get 7
                              call 57
                              local.get 2
                              call 58
                              local.get 3
                              i32.const 0
                              i32.store offset=68
                              i32.const 0
                              local.set 5
                            end
                            local.get 1
                            i32.const 1
                            i32.add
                            local.set 1
                            local.get 6
                            i32.const 1
                            i32.sub
                            local.tee 6
                            br_if 0 (;@12;)
                          end
                        end
                        local.get 3
                        i32.load offset=80
                        local.tee 1
                        local.get 5
                        i32.le_u
                        br_if 2 (;@8;)
                        local.get 3
                        i32.load offset=76
                        local.tee 2
                        local.get 5
                        i32.add
                        i32.const 128
                        i32.store8
                        local.get 3
                        local.get 5
                        i32.const 1
                        i32.add
                        local.tee 6
                        i32.store offset=68
                        local.get 5
                        i32.const 29
                        i32.le_u
                        if  ;; label = @11
                          loop  ;; label = @12
                            local.get 1
                            local.get 6
                            i32.eq
                            br_if 5 (;@7;)
                            local.get 2
                            local.get 6
                            i32.add
                            i32.const 0
                            i32.store8
                            local.get 6
                            i32.const 1
                            i32.add
                            local.tee 6
                            i32.const 31
                            i32.ne
                            br_if 0 (;@12;)
                          end
                          local.get 3
                          local.get 6
                          i32.store offset=68
                        end
                        local.get 1
                        i32.const 30
                        i32.le_u
                        br_if 8 (;@2;)
                        local.get 3
                        i32.load8_u offset=48
                        if  ;; label = @11
                          local.get 7
                          call 7
                          local.get 7
                          call 2
                          local.get 3
                          i32.const 0
                          i32.store8 offset=48
                        end
                        local.get 7
                        local.get 2
                        i32.const 31
                        i32.const 1
                        call 9
                        local.get 3
                        i32.const 8
                        i32.add
                        local.tee 1
                        local.get 7
                        call 57
                        local.get 1
                        call 58
                        local.get 3
                        i32.const 96
                        i32.add
                        local.tee 2
                        i32.const 8
                        i32.add
                        local.tee 5
                        i32.const 0
                        i32.store8
                        local.get 3
                        i64.const 0
                        i64.store offset=96
                        local.get 2
                        call 2
                        local.get 3
                        i32.const 128
                        i32.add
                        local.tee 2
                        i32.const 8
                        i32.add
                        local.tee 6
                        local.get 5
                        i64.load
                        i64.store
                        local.get 3
                        local.get 3
                        i64.load offset=96
                        i64.store offset=128
                        local.get 2
                        local.get 1
                        call 18
                        local.get 3
                        i32.load8_u offset=16
                        if  ;; label = @11
                          local.get 2
                          local.get 1
                          call 19
                          local.get 3
                          i32.const 1
                          i32.store8 offset=136
                        end
                        local.get 0
                        local.get 3
                        i64.load offset=128
                        i64.store
                        local.get 0
                        i32.const 8
                        i32.add
                        local.get 6
                        i64.load
                        i64.store
                        local.get 3
                        i32.const 8
                        i32.add
                        call 7
                        local.get 3
                        i32.const 24
                        i32.add
                        call 7
                        local.get 3
                        i32.load offset=72
                        if  ;; label = @11
                          local.get 3
                          i32.load offset=76
                          call 98
                        end
                        local.get 7
                        call 7
                        local.get 3
                        i32.load offset=88
                        local.set 0
                        local.get 3
                        i32.load offset=92
                        local.tee 5
                        if  ;; label = @11
                          local.get 0
                          local.set 4
                          loop  ;; label = @12
                            local.get 4
                            call 7
                            local.get 4
                            i32.const 16
                            i32.add
                            local.set 4
                            local.get 5
                            i32.const 1
                            i32.sub
                            local.tee 5
                            br_if 0 (;@12;)
                          end
                        end
                        local.get 3
                        i32.load offset=84
                        if  ;; label = @11
                          local.get 0
                          call 98
                        end
                        local.get 3
                        i32.const 144
                        i32.add
                        global.set 0
                        return
                      end
                      local.get 4
                      local.get 5
                      i32.add
                      local.set 4
                    end
                    local.get 3
                    local.get 4
                    i32.store offset=20
                    local.get 3
                    i32.const 66
                    i32.store offset=16
                    local.get 3
                    local.get 9
                    i32.store offset=12
                    local.get 3
                    i32.const 67
                    i32.store offset=8
                    global.get 0
                    i32.const -64
                    i32.add
                    local.tee 0
                    global.set 0
                    local.get 0
                    i32.const 28
                    i32.store offset=12
                    local.get 0
                    i32.const 1049600
                    i32.store offset=8
                    local.get 0
                    i32.const 1049584
                    i32.store offset=20
                    local.get 0
                    local.get 3
                    i32.const 8
                    i32.add
                    i32.store offset=16
                    local.get 0
                    i32.const 2
                    i32.store offset=28
                    local.get 0
                    i32.const 1048936
                    i32.store offset=24
                    local.get 0
                    i64.const 2
                    i64.store offset=36 align=4
                    local.get 0
                    local.get 0
                    i32.const 16
                    i32.add
                    i64.extend_i32_u
                    i64.const 12884901888
                    i64.or
                    i64.store offset=56
                    local.get 0
                    local.get 0
                    i32.const 8
                    i32.add
                    i64.extend_i32_u
                    i64.const 21474836480
                    i64.or
                    i64.store offset=48
                    local.get 0
                    local.get 0
                    i32.const 48
                    i32.add
                    i32.store offset=32
                    local.get 0
                    i32.const 24
                    i32.add
                    i32.const 1049712
                    call 39
                    unreachable
                  end
                  local.get 5
                  local.get 1
                  i32.const 1055708
                  call 27
                  unreachable
                end
                local.get 3
                local.get 6
                i32.store offset=68
                local.get 6
                local.get 1
                i32.const 1055740
                call 27
                unreachable
              end
              i32.const 31
              call 29
              unreachable
            end
            local.get 4
            local.get 2
            i32.const 1055692
            call 32
            unreachable
          end
          local.get 5
          local.get 2
          i32.const 1055660
          call 27
          unreachable
        end
        i32.const 31
        local.get 2
        i32.const 1055676
        call 32
        unreachable
      end
      i32.const 31
      local.get 1
      i32.const 1055724
      call 32
      unreachable
    end
    i32.const 2048
    call 29
    unreachable)
  (func (;34;) (type 3) (param i32 i32)
    (local i32 i32 i32 i64)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    i32.const 32
    i32.add
    local.tee 3
    i32.const 8
    i32.add
    local.tee 4
    i32.const 0
    i32.store8
    local.get 2
    i64.const 0
    i64.store offset=32
    local.get 3
    call 2
    local.get 2
    i32.const 8
    i32.add
    local.get 4
    i64.load
    local.tee 5
    i64.store
    local.get 2
    local.get 2
    i64.load offset=32
    i64.store
    block  ;; label = @1
      local.get 5
      i32.wrap_i64
      i32.const 255
      i32.and
      i32.eqz
      if  ;; label = @2
        local.get 2
        local.get 0
        local.get 1
        call 15
        br 1 (;@1;)
      end
      local.get 4
      i32.const 0
      i32.store8
      local.get 2
      i64.const 0
      i64.store offset=32
      local.get 2
      i32.const 32
      i32.add
      call 2
      local.get 2
      i32.const 16
      i32.add
      local.tee 3
      i32.const 8
      i32.add
      local.get 4
      i64.load
      i64.store
      local.get 2
      local.get 2
      i64.load offset=32
      i64.store offset=16
      local.get 3
      local.get 0
      local.get 1
      call 15
      local.get 2
      i32.const 0
      i32.store8 offset=8
      local.get 2
      i64.load offset=16
      local.set 5
      local.get 2
      local.get 2
      i64.load
      i64.store offset=16
      local.get 2
      local.get 5
      i64.store
      local.get 3
      call 7
    end
    local.get 2
    local.get 0
    local.get 1
    call 16
    local.get 1
    i32.const 1
    i32.store8 offset=8
    local.get 0
    i32.const 1
    i32.store8 offset=8
    local.get 0
    i64.load
    local.set 5
    local.get 0
    local.get 2
    i64.load
    i64.store
    local.get 2
    i32.const 1
    i32.store8 offset=8
    local.get 2
    local.get 5
    i64.store
    local.get 2
    call 7
    local.get 2
    i32.const 48
    i32.add
    global.set 0)
  (func (;35;) (type 11) (result i32)
    (local i64 i64 i64 i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 3
    global.set 0
    block  ;; label = @1
      i32.const 1057808
      i64.load
      local.tee 1
      i64.eqz
      if  ;; label = @2
        i32.const 1057816
        i64.load
        local.set 0
        loop  ;; label = @3
          local.get 0
          i64.const -1
          i64.eq
          br_if 2 (;@1;)
          i32.const 1057816
          i64.load
          local.tee 2
          local.get 0
          i64.eq
          local.set 4
          i32.const 1057816
          local.get 0
          i64.const 1
          i64.add
          local.tee 1
          local.get 2
          local.get 4
          select
          i64.store
          local.get 2
          local.set 0
          local.get 4
          i32.eqz
          br_if 0 (;@3;)
        end
        i32.const 1057808
        local.get 1
        i64.store
      end
      i32.const 1057824
      local.get 1
      i64.store
      call 26
      i32.const 1057765
      i32.load8_u
      i32.const 3
      i32.ne
      if  ;; label = @2
        local.get 3
        i32.const 1
        i32.store8 offset=15
        local.get 3
        i32.const 15
        i32.add
        call 36
      end
      local.get 3
      i32.const 16
      i32.add
      global.set 0
      i32.const 0
      return
    end
    call 37
    unreachable)
  (func (;36;) (type 2) (param i32)
    (local i32 i32 i64 i64 i64)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 1
    global.set 0
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    i32.const 1057765
                    i32.load8_u
                    i32.const 1
                    i32.sub
                    br_table 2 (;@6;) 6 (;@2;) 1 (;@7;) 0 (;@8;)
                  end
                  i32.const 1057765
                  i32.const 2
                  i32.store8
                  local.get 0
                  i32.load8_u
                  local.get 0
                  i32.const 0
                  i32.store8
                  i32.const 1
                  i32.ne
                  br_if 4 (;@3;)
                  local.get 1
                  i32.const 0
                  i32.store8 offset=8
                  block  ;; label = @8
                    i32.const 1057792
                    i32.load8_u
                    i32.eqz
                    local.tee 0
                    if  ;; label = @9
                      local.get 1
                      i32.const 8
                      i32.add
                      local.set 2
                      local.get 0
                      if  ;; label = @10
                        i32.const 1057768
                        i64.const 0
                        i64.store
                        local.get 2
                        i32.const 1
                        i32.store8
                        i32.const 1057792
                        i32.const 1
                        i32.store8
                        i32.const 1057776
                        i32.const 0
                        i32.store
                        i32.const 1057780
                        i32.const 0
                        i32.store8
                        i32.const 1057784
                        i32.const 0
                        i32.store
                      end
                      local.get 1
                      i32.load8_u offset=8
                      i32.const 1
                      i32.and
                      br_if 1 (;@8;)
                    end
                    i32.const 1057808
                    i64.load
                    local.tee 3
                    i64.eqz
                    if  ;; label = @9
                      i32.const 1057816
                      i64.load
                      local.set 4
                      loop  ;; label = @10
                        local.get 4
                        i64.const -1
                        i64.eq
                        br_if 5 (;@5;)
                        i32.const 1057816
                        i64.load
                        local.tee 5
                        local.get 4
                        i64.eq
                        local.set 0
                        i32.const 1057816
                        local.get 4
                        i64.const 1
                        i64.add
                        local.tee 3
                        local.get 5
                        local.get 0
                        select
                        i64.store
                        local.get 5
                        local.set 4
                        local.get 0
                        i32.eqz
                        br_if 0 (;@10;)
                      end
                      i32.const 1057808
                      local.get 3
                      i64.store
                    end
                    block  ;; label = @9
                      i32.const 1057768
                      i64.load
                      local.get 3
                      i64.ne
                      if  ;; label = @10
                        i32.const 1057780
                        i32.load8_u
                        i32.const 1
                        local.set 0
                        i32.const 1057780
                        i32.const 1
                        i32.store8
                        br_if 2 (;@8;)
                        i32.const 1057768
                        local.get 3
                        i64.store
                        br 1 (;@9;)
                      end
                      i32.const 1057776
                      i32.load
                      local.tee 0
                      i32.const -1
                      i32.eq
                      br_if 1 (;@8;)
                      local.get 0
                      i32.const 1
                      i32.add
                      local.set 0
                    end
                    i32.const 1057776
                    local.get 0
                    i32.store
                    i32.const 1057784
                    i32.load
                    br_if 4 (;@4;)
                    i32.const 1057776
                    local.get 0
                    i32.const 1
                    i32.sub
                    local.tee 0
                    i32.store
                    local.get 0
                    br_if 0 (;@8;)
                    i32.const 1057768
                    i64.const 0
                    i64.store
                    i32.const 1057780
                    i32.const 0
                    i32.store8
                  end
                  i32.const 1057765
                  i32.const 3
                  i32.store8
                end
                local.get 1
                i32.const 32
                i32.add
                global.set 0
                return
              end
              local.get 1
              i32.const 0
              i32.store offset=24
              local.get 1
              i32.const 1
              i32.store offset=12
              local.get 1
              i32.const 1057608
              i32.store offset=8
              br 4 (;@1;)
            end
            call 37
            unreachable
          end
          global.get 0
          i32.const 48
          i32.sub
          local.tee 0
          global.set 0
          local.get 0
          i32.const 1
          i32.store offset=12
          local.get 0
          i32.const 1048676
          i32.store offset=8
          local.get 0
          i64.const 1
          i64.store offset=20 align=4
          local.get 0
          local.get 0
          i32.const 47
          i32.add
          i64.extend_i32_u
          i64.const 25769803776
          i64.or
          i64.store offset=32
          local.get 0
          local.get 0
          i32.const 32
          i32.add
          i32.store offset=16
          local.get 0
          i32.const 8
          i32.add
          i32.const 1056396
          call 39
          unreachable
        end
        global.get 0
        i32.const 32
        i32.sub
        local.tee 0
        global.set 0
        local.get 0
        i32.const 0
        i32.store offset=16
        local.get 0
        i32.const 1
        i32.store offset=4
        local.get 0
        i64.const 4
        i64.store offset=8 align=4
        local.get 0
        i32.const 43
        i32.store offset=28
        local.get 0
        i32.const 1048685
        i32.store offset=24
        local.get 0
        local.get 0
        i32.const 24
        i32.add
        i32.store
        local.get 0
        i32.const 1056896
        call 39
        unreachable
      end
      local.get 1
      i32.const 0
      i32.store offset=24
      local.get 1
      i32.const 1
      i32.store offset=12
      local.get 1
      i32.const 1057672
      i32.store offset=8
    end
    local.get 1
    i64.const 4
    i64.store offset=16 align=4
    local.get 1
    i32.const 8
    i32.add
    i32.const 1056164
    call 39
    unreachable)
  (func (;37;) (type 8)
    (local i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 0
    global.set 0
    local.get 0
    i32.const 0
    i32.store offset=24
    local.get 0
    i32.const 1
    i32.store offset=12
    local.get 0
    i32.const 1056264
    i32.store offset=8
    local.get 0
    i64.const 4
    i64.store offset=16 align=4
    local.get 0
    i32.const 8
    i32.add
    i32.const 1056272
    call 39
    unreachable)
  (func (;38;) (type 4) (param i32 i32 i32)
    local.get 0
    if  ;; label = @1
      local.get 1
      call 29
      unreachable
    end
    local.get 2
    call 28
    unreachable)
  (func (;39;) (type 3) (param i32 i32)
    (local i32 i32 i64)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    i32.const 1
    i32.store16 offset=12
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
    local.get 0
    i32.store offset=4
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    local.get 2
    i32.const 4
    i32.add
    local.tee 0
    i64.load align=4
    local.set 4
    local.get 1
    local.get 0
    i32.store offset=12
    local.get 1
    local.get 4
    i64.store offset=4 align=4
    global.get 0
    i32.const 16
    i32.sub
    local.tee 0
    global.set 0
    local.get 1
    i32.const 4
    i32.add
    local.tee 1
    i32.load
    local.tee 2
    i32.load offset=12
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.load offset=4
            br_table 0 (;@4;) 1 (;@3;) 2 (;@2;)
          end
          local.get 3
          br_if 1 (;@2;)
          i32.const 1
          local.set 2
          i32.const 0
          local.set 3
          br 2 (;@1;)
        end
        local.get 3
        br_if 0 (;@2;)
        local.get 2
        i32.load
        local.tee 2
        i32.load offset=4
        local.set 3
        local.get 2
        i32.load
        local.set 2
        br 1 (;@1;)
      end
      local.get 0
      i32.const -2147483648
      i32.store
      local.get 0
      local.get 1
      i32.store offset=12
      local.get 0
      i32.const 1057340
      local.get 1
      i32.load offset=4
      local.get 1
      i32.load offset=8
      local.tee 0
      i32.load8_u offset=8
      local.get 0
      i32.load8_u offset=9
      call 61
      unreachable
    end
    local.get 0
    local.get 3
    i32.store offset=4
    local.get 0
    local.get 2
    i32.store
    local.get 0
    i32.const 1057312
    local.get 1
    i32.load offset=4
    local.get 1
    i32.load offset=8
    local.tee 0
    i32.load8_u offset=8
    local.get 0
    i32.load8_u offset=9
    call 61
    unreachable)
  (func (;40;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 4
    global.set 0
    i32.const 10
    local.set 2
    local.get 0
    i32.load
    local.tee 7
    local.tee 3
    i32.const 1000
    i32.ge_u
    if  ;; label = @1
      local.get 3
      local.set 0
      loop  ;; label = @2
        local.get 4
        i32.const 6
        i32.add
        local.get 2
        i32.add
        local.tee 5
        i32.const 3
        i32.sub
        local.get 0
        local.get 0
        i32.const 10000
        i32.div_u
        local.tee 3
        i32.const 10000
        i32.mul
        i32.sub
        local.tee 6
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 8
        i32.const 1
        i32.shl
        local.tee 9
        i32.const 1048991
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const 4
        i32.sub
        local.get 9
        i32.const 1048990
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const 1
        i32.sub
        local.get 6
        local.get 8
        i32.const 100
        i32.mul
        i32.sub
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        local.tee 6
        i32.const 1048991
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const 2
        i32.sub
        local.get 6
        i32.const 1048990
        i32.add
        i32.load8_u
        i32.store8
        local.get 2
        i32.const 4
        i32.sub
        local.set 2
        local.get 0
        i32.const 9999999
        i32.gt_u
        local.get 3
        local.set 0
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 3
      i32.const 9
      i32.le_u
      if  ;; label = @2
        local.get 3
        local.set 0
        br 1 (;@1;)
      end
      local.get 2
      local.get 4
      i32.add
      i32.const 5
      i32.add
      local.get 3
      local.get 3
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 0
      i32.const 100
      i32.mul
      i32.sub
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      local.tee 3
      i32.const 1048991
      i32.add
      i32.load8_u
      i32.store8
      local.get 2
      i32.const 2
      i32.sub
      local.tee 2
      local.get 4
      i32.const 6
      i32.add
      i32.add
      local.get 3
      i32.const 1048990
      i32.add
      i32.load8_u
      i32.store8
    end
    local.get 0
    i32.eqz
    local.get 7
    i32.const 0
    i32.ne
    i32.and
    i32.eqz
    if  ;; label = @1
      local.get 2
      i32.const 1
      i32.sub
      local.tee 2
      local.get 4
      i32.const 6
      i32.add
      i32.add
      local.get 0
      i32.const 1
      i32.shl
      i32.const 30
      i32.and
      i32.const 1048991
      i32.add
      i32.load8_u
      i32.store8
    end
    local.get 1
    i32.const 1
    i32.const 0
    local.get 4
    i32.const 6
    i32.add
    local.get 2
    i32.add
    i32.const 10
    local.get 2
    i32.sub
    call 41
    local.get 4
    i32.const 16
    i32.add
    global.set 0)
  (func (;41;) (type 9) (param i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i64)
    local.get 0
    i32.load offset=8
    local.tee 5
    i32.const 2097152
    i32.and
    local.tee 6
    i32.const 21
    i32.shr_u
    local.get 4
    i32.add
    local.set 7
    block  ;; label = @1
      local.get 5
      i32.const 8388608
      i32.and
      i32.eqz
      if  ;; label = @2
        i32.const 0
        local.set 1
        br 1 (;@1;)
      end
      block (result i32)  ;; label = @2
        i32.const 0
        local.get 2
        i32.eqz
        br_if 0 (;@2;)
        drop
        local.get 1
        i32.load8_s
        i32.const -65
        i32.gt_s
        local.tee 8
        local.get 2
        i32.const 1
        i32.eq
        br_if 0 (;@2;)
        drop
        local.get 8
        local.get 1
        i32.load8_s offset=1
        i32.const -65
        i32.gt_s
        i32.add
      end
      local.get 7
      i32.add
      local.set 7
    end
    i32.const 43
    i32.const 1114112
    local.get 6
    select
    local.set 11
    block  ;; label = @1
      local.get 7
      local.get 0
      i32.load16_u offset=12
      local.tee 8
      i32.lt_u
      if  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 5
            i32.const 16777216
            i32.and
            i32.eqz
            if  ;; label = @5
              local.get 8
              local.get 7
              i32.sub
              local.set 8
              i32.const 0
              local.set 6
              i32.const 0
              local.set 7
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 5
                    i32.const 29
                    i32.shr_u
                    i32.const 3
                    i32.and
                    i32.const 1
                    i32.sub
                    br_table 0 (;@8;) 1 (;@7;) 0 (;@8;) 2 (;@6;)
                  end
                  local.get 8
                  local.set 7
                  br 1 (;@6;)
                end
                local.get 8
                i32.const 65534
                i32.and
                i32.const 1
                i32.shr_u
                local.set 7
              end
              local.get 5
              i32.const 2097151
              i32.and
              local.set 10
              local.get 0
              i32.load offset=4
              local.set 9
              local.get 0
              i32.load
              local.set 0
              loop  ;; label = @6
                local.get 6
                i32.const 65535
                i32.and
                local.get 7
                i32.const 65535
                i32.and
                i32.ge_u
                br_if 2 (;@4;)
                i32.const 1
                local.set 5
                local.get 6
                i32.const 1
                i32.add
                local.set 6
                local.get 0
                local.get 10
                local.get 9
                i32.load offset=16
                call_indirect (type 0)
                i32.eqz
                br_if 0 (;@6;)
              end
              br 4 (;@1;)
            end
            local.get 0
            local.get 0
            i64.load offset=8 align=4
            local.tee 12
            i32.wrap_i64
            i32.const -1612709888
            i32.and
            i32.const 536870960
            i32.or
            i32.store offset=8
            i32.const 1
            local.set 5
            local.get 0
            i32.load
            local.tee 9
            local.get 0
            i32.load offset=4
            local.tee 10
            local.get 11
            local.get 1
            local.get 2
            call 42
            br_if 3 (;@1;)
            i32.const 0
            local.set 6
            local.get 8
            local.get 7
            i32.sub
            i32.const 65535
            i32.and
            local.set 1
            loop  ;; label = @5
              local.get 6
              i32.const 65535
              i32.and
              local.get 1
              i32.ge_u
              br_if 2 (;@3;)
              local.get 6
              i32.const 1
              i32.add
              local.set 6
              local.get 9
              i32.const 48
              local.get 10
              i32.load offset=16
              call_indirect (type 0)
              i32.eqz
              br_if 0 (;@5;)
            end
            br 3 (;@1;)
          end
          i32.const 1
          local.set 5
          local.get 0
          local.get 9
          local.get 11
          local.get 1
          local.get 2
          call 42
          br_if 2 (;@1;)
          local.get 0
          local.get 3
          local.get 4
          local.get 9
          i32.load offset=12
          call_indirect (type 1)
          br_if 2 (;@1;)
          local.get 8
          local.get 7
          i32.sub
          i32.const 65535
          i32.and
          local.set 1
          i32.const 0
          local.set 6
          loop  ;; label = @4
            local.get 1
            local.get 6
            i32.const 65535
            i32.and
            i32.le_u
            if  ;; label = @5
              i32.const 0
              return
            end
            local.get 6
            i32.const 1
            i32.add
            local.set 6
            local.get 0
            local.get 10
            local.get 9
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 0 (;@4;)
          end
          br 2 (;@1;)
        end
        local.get 9
        local.get 3
        local.get 4
        local.get 10
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
        local.get 0
        local.get 12
        i64.store offset=8 align=4
        i32.const 0
        return
      end
      i32.const 1
      local.set 5
      local.get 0
      i32.load
      local.tee 7
      local.get 0
      i32.load offset=4
      local.tee 0
      local.get 11
      local.get 1
      local.get 2
      call 42
      br_if 0 (;@1;)
      local.get 7
      local.get 3
      local.get 4
      local.get 0
      i32.load offset=12
      call_indirect (type 1)
      local.set 5
    end
    local.get 5)
  (func (;42;) (type 9) (param i32 i32 i32 i32 i32) (result i32)
    block  ;; label = @1
      local.get 2
      i32.const 1114112
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      local.get 1
      i32.load offset=16
      call_indirect (type 0)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    local.get 3
    i32.eqz
    if  ;; label = @1
      i32.const 0
      return
    end
    local.get 0
    local.get 3
    local.get 4
    local.get 1
    i32.load offset=12
    call_indirect (type 1))
  (func (;43;) (type 4) (param i32 i32 i32)
    (local i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1049252
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=40
    local.get 3
    local.get 3
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call 39
    unreachable)
  (func (;44;) (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load offset=8
        local.tee 10
        i32.const 402653184
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        block  ;; label = @3
          block (result i32)  ;; label = @4
            block  ;; label = @5
              local.get 10
              i32.const 268435456
              i32.and
              if  ;; label = @6
                local.get 0
                i32.load16_u offset=14
                local.tee 5
                br_if 1 (;@5;)
                i32.const 0
                local.set 2
                i32.const 0
                br 2 (;@4;)
              end
              local.get 2
              i32.const 16
              i32.ge_u
              if  ;; label = @6
                local.get 1
                local.get 1
                i32.const 3
                i32.add
                i32.const -4
                i32.and
                local.tee 3
                i32.sub
                local.tee 7
                local.get 2
                i32.add
                local.tee 11
                i32.const 3
                i32.and
                local.set 8
                block  ;; label = @7
                  local.get 1
                  local.get 3
                  i32.eq
                  local.tee 12
                  br_if 0 (;@7;)
                  block  ;; label = @8
                    local.get 7
                    i32.const -4
                    i32.gt_u
                    if  ;; label = @9
                      br 1 (;@8;)
                    end
                    loop  ;; label = @9
                      local.get 4
                      local.get 1
                      local.get 6
                      i32.add
                      local.tee 9
                      i32.load8_s
                      i32.const -65
                      i32.gt_s
                      i32.add
                      local.get 9
                      i32.const 1
                      i32.add
                      i32.load8_s
                      i32.const -65
                      i32.gt_s
                      i32.add
                      local.get 9
                      i32.const 2
                      i32.add
                      i32.load8_s
                      i32.const -65
                      i32.gt_s
                      i32.add
                      local.get 9
                      i32.const 3
                      i32.add
                      i32.load8_s
                      i32.const -65
                      i32.gt_s
                      i32.add
                      local.set 4
                      local.get 6
                      i32.const 4
                      i32.add
                      local.tee 6
                      br_if 0 (;@9;)
                    end
                  end
                  local.get 12
                  br_if 0 (;@7;)
                  local.get 1
                  local.get 6
                  i32.add
                  local.set 6
                  loop  ;; label = @8
                    local.get 4
                    local.get 6
                    i32.load8_s
                    i32.const -65
                    i32.gt_s
                    i32.add
                    local.set 4
                    local.get 6
                    i32.const 1
                    i32.add
                    local.set 6
                    local.get 7
                    i32.const 1
                    i32.add
                    local.tee 7
                    br_if 0 (;@8;)
                  end
                end
                block  ;; label = @7
                  local.get 8
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 3
                  local.get 11
                  i32.const -4
                  i32.and
                  i32.add
                  local.tee 6
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  local.set 5
                  local.get 8
                  i32.const 1
                  i32.eq
                  br_if 0 (;@7;)
                  local.get 5
                  local.get 6
                  i32.load8_s offset=1
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.set 5
                  local.get 8
                  i32.const 2
                  i32.eq
                  br_if 0 (;@7;)
                  local.get 5
                  local.get 6
                  i32.load8_s offset=2
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.set 5
                end
                local.get 11
                i32.const 2
                i32.shr_u
                local.set 7
                local.get 4
                local.get 5
                i32.add
                local.set 5
                loop  ;; label = @7
                  local.get 3
                  local.set 6
                  local.get 7
                  i32.eqz
                  br_if 4 (;@3;)
                  i32.const 192
                  local.get 7
                  local.get 7
                  i32.const 192
                  i32.ge_u
                  select
                  local.tee 8
                  i32.const 3
                  i32.and
                  local.set 9
                  local.get 8
                  i32.const 2
                  i32.shl
                  local.set 11
                  i32.const 0
                  local.set 4
                  local.get 7
                  i32.const 4
                  i32.ge_u
                  if  ;; label = @8
                    local.get 3
                    local.get 11
                    i32.const 1008
                    i32.and
                    i32.add
                    local.set 12
                    loop  ;; label = @9
                      local.get 4
                      local.get 3
                      i32.load
                      local.tee 4
                      i32.const -1
                      i32.xor
                      i32.const 7
                      i32.shr_u
                      local.get 4
                      i32.const 6
                      i32.shr_u
                      i32.or
                      i32.const 16843009
                      i32.and
                      i32.add
                      local.get 3
                      i32.const 4
                      i32.add
                      i32.load
                      local.tee 4
                      i32.const -1
                      i32.xor
                      i32.const 7
                      i32.shr_u
                      local.get 4
                      i32.const 6
                      i32.shr_u
                      i32.or
                      i32.const 16843009
                      i32.and
                      i32.add
                      local.get 3
                      i32.const 8
                      i32.add
                      i32.load
                      local.tee 4
                      i32.const -1
                      i32.xor
                      i32.const 7
                      i32.shr_u
                      local.get 4
                      i32.const 6
                      i32.shr_u
                      i32.or
                      i32.const 16843009
                      i32.and
                      i32.add
                      local.get 3
                      i32.const 12
                      i32.add
                      i32.load
                      local.tee 4
                      i32.const -1
                      i32.xor
                      i32.const 7
                      i32.shr_u
                      local.get 4
                      i32.const 6
                      i32.shr_u
                      i32.or
                      i32.const 16843009
                      i32.and
                      i32.add
                      local.set 4
                      local.get 12
                      local.get 3
                      i32.const 16
                      i32.add
                      local.tee 3
                      i32.ne
                      br_if 0 (;@9;)
                    end
                  end
                  local.get 7
                  local.get 8
                  i32.sub
                  local.set 7
                  local.get 6
                  local.get 11
                  i32.add
                  local.set 3
                  local.get 4
                  i32.const 8
                  i32.shr_u
                  i32.const 16711935
                  i32.and
                  local.get 4
                  i32.const 16711935
                  i32.and
                  i32.add
                  i32.const 65537
                  i32.mul
                  i32.const 16
                  i32.shr_u
                  local.get 5
                  i32.add
                  local.set 5
                  local.get 9
                  i32.eqz
                  br_if 0 (;@7;)
                end
                block (result i32)  ;; label = @7
                  local.get 6
                  local.get 8
                  i32.const 252
                  i32.and
                  i32.const 2
                  i32.shl
                  i32.add
                  local.tee 4
                  i32.load
                  local.tee 3
                  i32.const -1
                  i32.xor
                  i32.const 7
                  i32.shr_u
                  local.get 3
                  i32.const 6
                  i32.shr_u
                  i32.or
                  i32.const 16843009
                  i32.and
                  local.tee 3
                  local.get 9
                  i32.const 1
                  i32.eq
                  br_if 0 (;@7;)
                  drop
                  local.get 3
                  local.get 4
                  i32.load offset=4
                  local.tee 3
                  i32.const -1
                  i32.xor
                  i32.const 7
                  i32.shr_u
                  local.get 3
                  i32.const 6
                  i32.shr_u
                  i32.or
                  i32.const 16843009
                  i32.and
                  i32.add
                  local.tee 3
                  local.get 9
                  i32.const 2
                  i32.eq
                  br_if 0 (;@7;)
                  drop
                  local.get 4
                  i32.load offset=8
                  local.tee 4
                  i32.const -1
                  i32.xor
                  i32.const 7
                  i32.shr_u
                  local.get 4
                  i32.const 6
                  i32.shr_u
                  i32.or
                  i32.const 16843009
                  i32.and
                  local.get 3
                  i32.add
                end
                local.tee 3
                i32.const 8
                i32.shr_u
                i32.const 459007
                i32.and
                local.get 3
                i32.const 16711935
                i32.and
                i32.add
                i32.const 65537
                i32.mul
                i32.const 16
                i32.shr_u
                local.get 5
                i32.add
                local.set 5
                br 3 (;@3;)
              end
              local.get 2
              i32.eqz
              if  ;; label = @6
                i32.const 0
                local.set 2
                br 3 (;@3;)
              end
              local.get 2
              i32.const 3
              i32.and
              local.set 6
              block  ;; label = @6
                local.get 2
                i32.const 4
                i32.lt_u
                if  ;; label = @7
                  br 1 (;@6;)
                end
                local.get 2
                i32.const 12
                i32.and
                local.set 7
                loop  ;; label = @7
                  local.get 5
                  local.get 1
                  local.get 4
                  i32.add
                  local.tee 3
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.get 3
                  i32.const 1
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.get 3
                  i32.const 2
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.get 3
                  i32.const 3
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.set 5
                  local.get 7
                  local.get 4
                  i32.const 4
                  i32.add
                  local.tee 4
                  i32.ne
                  br_if 0 (;@7;)
                end
              end
              local.get 6
              i32.eqz
              br_if 2 (;@3;)
              local.get 1
              local.get 4
              i32.add
              local.set 3
              loop  ;; label = @6
                local.get 5
                local.get 3
                i32.load8_s
                i32.const -65
                i32.gt_s
                i32.add
                local.set 5
                local.get 3
                i32.const 1
                i32.add
                local.set 3
                local.get 6
                i32.const 1
                i32.sub
                local.tee 6
                br_if 0 (;@6;)
              end
              br 2 (;@3;)
            end
            local.get 1
            local.get 2
            i32.add
            local.set 7
            i32.const 0
            local.set 2
            local.get 1
            local.set 4
            block  ;; label = @5
              loop  ;; label = @6
                local.get 4
                local.get 7
                i32.eq
                br_if 1 (;@5;)
                block (result i32)  ;; label = @7
                  local.get 4
                  local.tee 3
                  i32.load8_s
                  local.tee 4
                  i32.const 0
                  i32.ge_s
                  if  ;; label = @8
                    local.get 3
                    i32.const 1
                    i32.add
                    br 1 (;@7;)
                  end
                  local.get 3
                  i32.const 2
                  i32.add
                  local.get 4
                  i32.const -32
                  i32.lt_u
                  br_if 0 (;@7;)
                  drop
                  local.get 3
                  i32.const 3
                  i32.add
                  local.get 4
                  i32.const -16
                  i32.lt_u
                  br_if 0 (;@7;)
                  drop
                  local.get 3
                  i32.const 4
                  i32.add
                end
                local.tee 4
                local.get 3
                i32.sub
                local.get 2
                i32.add
                local.set 2
                local.get 5
                local.get 6
                i32.const 1
                i32.add
                local.tee 6
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              br 1 (;@4;)
            end
            local.get 5
            local.get 6
            i32.sub
          end
          local.set 3
          local.get 5
          local.get 3
          i32.sub
          local.set 5
        end
        local.get 0
        i32.load16_u offset=12
        local.tee 4
        local.get 5
        i32.le_u
        br_if 0 (;@2;)
        local.get 4
        local.get 5
        i32.sub
        local.set 6
        i32.const 0
        local.set 3
        i32.const 0
        local.set 5
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 10
              i32.const 29
              i32.shr_u
              i32.const 3
              i32.and
              i32.const 1
              i32.sub
              br_table 0 (;@5;) 1 (;@4;) 2 (;@3;)
            end
            local.get 6
            local.set 5
            br 1 (;@3;)
          end
          local.get 6
          i32.const 65534
          i32.and
          i32.const 1
          i32.shr_u
          local.set 5
        end
        local.get 10
        i32.const 2097151
        i32.and
        local.set 10
        local.get 0
        i32.load offset=4
        local.set 7
        local.get 0
        i32.load
        local.set 0
        loop  ;; label = @3
          local.get 3
          i32.const 65535
          i32.and
          local.get 5
          i32.const 65535
          i32.and
          i32.lt_u
          if  ;; label = @4
            i32.const 1
            local.set 4
            local.get 3
            i32.const 1
            i32.add
            local.set 3
            local.get 0
            local.get 10
            local.get 7
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 1 (;@3;)
            br 3 (;@1;)
          end
        end
        i32.const 1
        local.set 4
        local.get 0
        local.get 1
        local.get 2
        local.get 7
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
        local.get 6
        local.get 5
        i32.sub
        i32.const 65535
        i32.and
        local.set 1
        i32.const 0
        local.set 3
        loop  ;; label = @3
          local.get 1
          local.get 3
          i32.const 65535
          i32.and
          i32.le_u
          if  ;; label = @4
            i32.const 0
            return
          end
          local.get 3
          i32.const 1
          i32.add
          local.set 3
          local.get 0
          local.get 10
          local.get 7
          i32.load offset=16
          call_indirect (type 0)
          i32.eqz
          br_if 0 (;@3;)
        end
        br 1 (;@1;)
      end
      local.get 0
      i32.load
      local.get 1
      local.get 2
      local.get 0
      i32.load offset=4
      i32.load offset=12
      call_indirect (type 1)
      local.set 4
    end
    local.get 4)
  (func (;45;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 1
    local.get 0
    i32.load offset=4
    i32.load offset=12
    call_indirect (type 0))
  (func (;46;) (type 0) (param i32 i32) (result i32)
    local.get 1
    i32.load
    local.get 1
    i32.load offset=4
    local.get 0
    call 48)
  (func (;47;) (type 0) (param i32 i32) (result i32)
    local.get 1
    local.get 0
    i32.load
    local.get 0
    i32.load offset=4
    call 44)
  (func (;48;) (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i64.const 3758096416
    i64.store offset=8 align=4
    block (result i32)  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            i32.load offset=16
            local.tee 9
            if  ;; label = @5
              local.get 2
              i32.load offset=20
              local.tee 0
              br_if 1 (;@4;)
              br 2 (;@3;)
            end
            local.get 2
            i32.load offset=12
            local.tee 0
            i32.eqz
            br_if 1 (;@3;)
            local.get 2
            i32.load offset=8
            local.tee 1
            local.get 0
            i32.const 3
            i32.shl
            i32.add
            local.set 4
            local.get 0
            i32.const 1
            i32.sub
            i32.const 536870911
            i32.and
            i32.const 1
            i32.add
            local.set 6
            local.get 2
            i32.load
            local.set 0
            loop  ;; label = @5
              block  ;; label = @6
                local.get 0
                i32.const 4
                i32.add
                i32.load
                local.tee 5
                i32.eqz
                br_if 0 (;@6;)
                local.get 3
                i32.load
                local.get 0
                i32.load
                local.get 5
                local.get 3
                i32.load offset=4
                i32.load offset=12
                call_indirect (type 1)
                i32.eqz
                br_if 0 (;@6;)
                i32.const 1
                br 5 (;@1;)
              end
              i32.const 1
              local.get 1
              i32.load
              local.get 3
              local.get 1
              i32.const 4
              i32.add
              i32.load
              call_indirect (type 0)
              br_if 4 (;@1;)
              drop
              local.get 0
              i32.const 8
              i32.add
              local.set 0
              local.get 1
              i32.const 8
              i32.add
              local.tee 1
              local.get 4
              i32.ne
              br_if 0 (;@5;)
            end
            br 2 (;@2;)
          end
          local.get 0
          i32.const 24
          i32.mul
          local.set 10
          local.get 0
          i32.const 1
          i32.sub
          i32.const 536870911
          i32.and
          i32.const 1
          i32.add
          local.set 6
          local.get 2
          i32.load offset=8
          local.set 4
          local.get 2
          i32.load
          local.set 0
          loop  ;; label = @4
            block  ;; label = @5
              local.get 0
              i32.const 4
              i32.add
              i32.load
              local.tee 1
              i32.eqz
              br_if 0 (;@5;)
              local.get 3
              i32.load
              local.get 0
              i32.load
              local.get 1
              local.get 3
              i32.load offset=4
              i32.load offset=12
              call_indirect (type 1)
              i32.eqz
              br_if 0 (;@5;)
              i32.const 1
              br 4 (;@1;)
            end
            i32.const 0
            local.set 7
            i32.const 0
            local.set 8
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  local.get 5
                  local.get 9
                  i32.add
                  local.tee 1
                  i32.const 8
                  i32.add
                  i32.load16_u
                  i32.const 1
                  i32.sub
                  br_table 1 (;@6;) 2 (;@5;) 0 (;@7;)
                end
                local.get 1
                i32.const 10
                i32.add
                i32.load16_u
                local.set 8
                br 1 (;@5;)
              end
              local.get 4
              local.get 1
              i32.const 12
              i32.add
              i32.load
              i32.const 3
              i32.shl
              i32.add
              i32.load16_u offset=4
              local.set 8
            end
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  local.get 1
                  i32.load16_u
                  i32.const 1
                  i32.sub
                  br_table 1 (;@6;) 2 (;@5;) 0 (;@7;)
                end
                local.get 1
                i32.const 2
                i32.add
                i32.load16_u
                local.set 7
                br 1 (;@5;)
              end
              local.get 4
              local.get 1
              i32.const 4
              i32.add
              i32.load
              i32.const 3
              i32.shl
              i32.add
              i32.load16_u offset=4
              local.set 7
            end
            local.get 3
            local.get 7
            i32.store16 offset=14
            local.get 3
            local.get 8
            i32.store16 offset=12
            local.get 3
            local.get 1
            i32.const 20
            i32.add
            i32.load
            i32.store offset=8
            i32.const 1
            local.get 4
            local.get 1
            i32.const 16
            i32.add
            i32.load
            i32.const 3
            i32.shl
            i32.add
            local.tee 1
            i32.load
            local.get 3
            local.get 1
            i32.load offset=4
            call_indirect (type 0)
            br_if 3 (;@1;)
            drop
            local.get 0
            i32.const 8
            i32.add
            local.set 0
            local.get 5
            i32.const 24
            i32.add
            local.tee 5
            local.get 10
            i32.ne
            br_if 0 (;@4;)
          end
          br 1 (;@2;)
        end
      end
      block  ;; label = @2
        local.get 6
        local.get 2
        i32.load offset=4
        i32.ge_u
        br_if 0 (;@2;)
        local.get 3
        i32.load
        local.get 2
        i32.load
        local.get 6
        i32.const 3
        i32.shl
        i32.add
        local.tee 0
        i32.load
        local.get 0
        i32.load offset=4
        local.get 3
        i32.load offset=4
        i32.load offset=12
        call_indirect (type 1)
        i32.eqz
        br_if 0 (;@2;)
        i32.const 1
        br 1 (;@1;)
      end
      i32.const 0
    end
    local.get 3
    i32.const 16
    i32.add
    global.set 0)
  (func (;49;) (type 0) (param i32 i32) (result i32)
    local.get 1
    i32.load
    i32.const 1048641
    i32.const 14
    local.get 1
    i32.load offset=4
    i32.load offset=12
    call_indirect (type 1))
  (func (;50;) (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 1
    i32.const 1
    i32.sub
    local.set 14
    local.get 0
    i32.load offset=4
    local.set 10
    local.get 0
    i32.load
    local.set 11
    local.get 0
    i32.load offset=8
    local.set 12
    block  ;; label = @1
      loop  ;; label = @2
        local.get 5
        br_if 1 (;@1;)
        block (result i32)  ;; label = @3
          block  ;; label = @4
            local.get 2
            local.get 3
            i32.lt_u
            br_if 0 (;@4;)
            loop  ;; label = @5
              local.get 1
              local.get 3
              i32.add
              local.set 5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 2
                    local.get 3
                    i32.sub
                    local.tee 7
                    i32.const 7
                    i32.le_u
                    if  ;; label = @9
                      local.get 2
                      local.get 3
                      i32.ne
                      br_if 1 (;@8;)
                      local.get 2
                      local.set 3
                      br 5 (;@4;)
                    end
                    block  ;; label = @9
                      local.get 5
                      i32.const 3
                      i32.add
                      i32.const -4
                      i32.and
                      local.tee 6
                      local.get 5
                      i32.sub
                      local.tee 4
                      if  ;; label = @10
                        i32.const 0
                        local.set 0
                        loop  ;; label = @11
                          local.get 0
                          local.get 5
                          i32.add
                          i32.load8_u
                          i32.const 10
                          i32.eq
                          br_if 5 (;@6;)
                          local.get 4
                          local.get 0
                          i32.const 1
                          i32.add
                          local.tee 0
                          i32.ne
                          br_if 0 (;@11;)
                        end
                        local.get 7
                        i32.const 8
                        i32.sub
                        local.tee 0
                        local.get 4
                        i32.ge_u
                        br_if 1 (;@9;)
                        br 3 (;@7;)
                      end
                      local.get 7
                      i32.const 8
                      i32.sub
                      local.set 0
                    end
                    loop  ;; label = @9
                      local.get 6
                      i32.load
                      local.tee 9
                      i32.const 16843008
                      local.get 9
                      i32.const 168430090
                      i32.xor
                      i32.sub
                      i32.or
                      local.get 6
                      i32.const 4
                      i32.add
                      i32.load
                      local.tee 9
                      i32.const 16843008
                      local.get 9
                      i32.const 168430090
                      i32.xor
                      i32.sub
                      i32.or
                      i32.and
                      i32.const -2139062144
                      i32.and
                      i32.const -2139062144
                      i32.ne
                      br_if 2 (;@7;)
                      local.get 6
                      i32.const 8
                      i32.add
                      local.set 6
                      local.get 0
                      local.get 4
                      i32.const 8
                      i32.add
                      local.tee 4
                      i32.ge_u
                      br_if 0 (;@9;)
                    end
                    br 1 (;@7;)
                  end
                  i32.const 0
                  local.set 0
                  loop  ;; label = @8
                    local.get 0
                    local.get 5
                    i32.add
                    i32.load8_u
                    i32.const 10
                    i32.eq
                    br_if 2 (;@6;)
                    local.get 7
                    local.get 0
                    i32.const 1
                    i32.add
                    local.tee 0
                    i32.ne
                    br_if 0 (;@8;)
                  end
                  local.get 2
                  local.set 3
                  br 3 (;@4;)
                end
                local.get 4
                local.get 7
                i32.eq
                if  ;; label = @7
                  local.get 2
                  local.set 3
                  br 3 (;@4;)
                end
                local.get 4
                local.get 5
                i32.add
                local.set 6
                local.get 2
                local.get 4
                i32.sub
                local.get 3
                i32.sub
                local.set 7
                i32.const 0
                local.set 0
                block  ;; label = @7
                  loop  ;; label = @8
                    local.get 0
                    local.get 6
                    i32.add
                    i32.load8_u
                    i32.const 10
                    i32.eq
                    br_if 1 (;@7;)
                    local.get 7
                    local.get 0
                    i32.const 1
                    i32.add
                    local.tee 0
                    i32.ne
                    br_if 0 (;@8;)
                  end
                  local.get 2
                  local.set 3
                  br 3 (;@4;)
                end
                local.get 0
                local.get 4
                i32.add
                local.set 0
              end
              local.get 0
              local.get 3
              i32.add
              local.tee 4
              i32.const 1
              i32.add
              local.set 3
              block  ;; label = @6
                local.get 2
                local.get 4
                i32.le_u
                br_if 0 (;@6;)
                local.get 0
                local.get 5
                i32.add
                i32.load8_u
                i32.const 10
                i32.ne
                br_if 0 (;@6;)
                i32.const 0
                local.set 5
                local.get 3
                local.tee 4
                br 3 (;@3;)
              end
              local.get 2
              local.get 3
              i32.ge_u
              br_if 0 (;@5;)
            end
          end
          local.get 2
          local.get 8
          i32.eq
          br_if 2 (;@1;)
          i32.const 1
          local.set 5
          local.get 8
          local.set 4
          local.get 2
        end
        local.set 0
        block  ;; label = @3
          local.get 12
          i32.load8_u
          if  ;; label = @4
            local.get 11
            i32.const 1048976
            i32.const 4
            local.get 10
            i32.load offset=12
            call_indirect (type 1)
            br_if 1 (;@3;)
          end
          local.get 0
          local.get 8
          i32.sub
          local.set 7
          i32.const 0
          local.set 6
          local.get 0
          local.get 8
          i32.ne
          if  ;; label = @4
            local.get 0
            local.get 14
            i32.add
            i32.load8_u
            i32.const 10
            i32.eq
            local.set 6
          end
          local.get 1
          local.get 8
          i32.add
          local.set 0
          local.get 12
          local.get 6
          i32.store8
          local.get 4
          local.set 8
          local.get 11
          local.get 0
          local.get 7
          local.get 10
          i32.load offset=12
          call_indirect (type 1)
          i32.eqz
          br_if 1 (;@2;)
        end
      end
      i32.const 1
      local.set 13
    end
    local.get 13)
  (func (;51;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 0
    i32.load
    local.set 3
    block  ;; label = @1
      local.get 0
      i32.load offset=8
      local.tee 0
      i32.load8_u
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      i32.const 1048976
      i32.const 4
      local.get 2
      i32.load offset=12
      call_indirect (type 1)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    local.get 0
    local.get 1
    i32.const 10
    i32.eq
    i32.store8
    local.get 3
    local.get 1
    local.get 2
    i32.load offset=16
    call_indirect (type 0))
  (func (;52;) (type 0) (param i32 i32) (result i32)
    local.get 1
    i32.load offset=4
    drop
    local.get 0
    i32.const 1048952
    local.get 1
    call 48)
  (func (;53;) (type 2) (param i32)
    local.get 0
    i32.load
    if  ;; label = @1
      local.get 0
      i32.load offset=4
      call 98
    end)
  (func (;54;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i64)
    global.get 0
    i32.const 160
    i32.sub
    local.tee 2
    global.set 0
    block  ;; label = @1
      local.get 1
      i32.load
      local.tee 4
      i32.const 1049352
      i32.const 8
      local.get 1
      i32.load offset=4
      local.tee 5
      i32.load offset=12
      local.tee 6
      call_indirect (type 1)
      if  ;; label = @2
        i32.const 1
        local.set 5
        br 1 (;@1;)
      end
      local.get 0
      i32.const 12
      i32.add
      local.set 3
      block  ;; label = @2
        local.get 1
        i32.load offset=8
        local.tee 7
        i32.const 8388608
        i32.and
        i32.eqz
        if  ;; label = @3
          i32.const 1
          local.set 5
          local.get 4
          i32.const 1048984
          i32.const 1
          local.get 6
          call_indirect (type 1)
          br_if 2 (;@1;)
          block  ;; label = @4
            local.get 7
            i32.const 33554432
            i32.and
            i32.eqz
            if  ;; label = @5
              local.get 7
              i32.const 67108864
              i32.and
              br_if 1 (;@4;)
              local.get 3
              local.get 1
              call 40
              i32.eqz
              br_if 3 (;@2;)
              br 4 (;@1;)
            end
            local.get 3
            i32.load
            local.set 4
            i32.const 129
            local.set 3
            loop  ;; label = @5
              local.get 2
              local.get 3
              i32.add
              i32.const 30
              i32.add
              local.get 4
              i32.const 15
              i32.and
              local.tee 6
              i32.const 48
              i32.or
              local.get 6
              i32.const 87
              i32.add
              local.get 6
              i32.const 10
              i32.lt_u
              select
              i32.store8
              local.get 3
              i32.const 1
              i32.sub
              local.set 3
              local.get 4
              i32.const 16
              i32.lt_u
              local.get 4
              i32.const 4
              i32.shr_u
              local.set 4
              i32.eqz
              br_if 0 (;@5;)
            end
            local.get 1
            i32.const 1048988
            i32.const 2
            local.get 2
            local.get 3
            i32.add
            i32.const 31
            i32.add
            i32.const 129
            local.get 3
            i32.sub
            call 41
            i32.eqz
            br_if 2 (;@2;)
            br 3 (;@1;)
          end
          local.get 3
          i32.load
          local.set 4
          i32.const 129
          local.set 3
          loop  ;; label = @4
            local.get 2
            local.get 3
            i32.add
            i32.const 30
            i32.add
            local.get 4
            i32.const 15
            i32.and
            local.tee 6
            i32.const 48
            i32.or
            local.get 6
            i32.const 55
            i32.add
            local.get 6
            i32.const 10
            i32.lt_u
            select
            i32.store8
            local.get 3
            i32.const 1
            i32.sub
            local.set 3
            local.get 4
            i32.const 15
            i32.gt_u
            local.get 4
            i32.const 4
            i32.shr_u
            local.set 4
            br_if 0 (;@4;)
          end
          local.get 1
          i32.const 1048988
          i32.const 2
          local.get 2
          local.get 3
          i32.add
          i32.const 31
          i32.add
          i32.const 129
          local.get 3
          i32.sub
          call 41
          i32.eqz
          br_if 1 (;@2;)
          br 2 (;@1;)
        end
        local.get 4
        i32.const 1048985
        i32.const 2
        local.get 6
        call_indirect (type 1)
        if  ;; label = @3
          i32.const 1
          local.set 5
          br 2 (;@1;)
        end
        local.get 2
        i32.const 1
        i32.store8 offset=15
        local.get 2
        local.get 5
        i32.store offset=4
        local.get 2
        local.get 4
        i32.store
        local.get 2
        i32.const 1048952
        i32.store offset=20
        local.get 2
        local.get 1
        i64.load offset=8 align=4
        local.tee 8
        i64.store offset=24 align=4
        local.get 2
        local.get 2
        i32.const 15
        i32.add
        i32.store offset=8
        local.get 2
        local.get 2
        i32.store offset=16
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 8
              i32.wrap_i64
              local.tee 4
              i32.const 33554432
              i32.and
              i32.eqz
              if  ;; label = @6
                local.get 4
                i32.const 67108864
                i32.and
                br_if 1 (;@5;)
                local.get 3
                local.get 2
                i32.const 16
                i32.add
                call 40
                i32.eqz
                br_if 3 (;@3;)
                br 2 (;@4;)
              end
              local.get 3
              i32.load
              local.set 4
              i32.const 129
              local.set 3
              loop  ;; label = @6
                local.get 2
                local.get 3
                i32.add
                i32.const 30
                i32.add
                local.get 4
                i32.const 15
                i32.and
                local.tee 5
                i32.const 48
                i32.or
                local.get 5
                i32.const 87
                i32.add
                local.get 5
                i32.const 10
                i32.lt_u
                select
                i32.store8
                local.get 3
                i32.const 1
                i32.sub
                local.set 3
                local.get 4
                i32.const 16
                i32.lt_u
                local.get 4
                i32.const 4
                i32.shr_u
                local.set 4
                i32.eqz
                br_if 0 (;@6;)
              end
              local.get 2
              i32.const 16
              i32.add
              i32.const 1048988
              i32.const 2
              local.get 2
              local.get 3
              i32.add
              i32.const 31
              i32.add
              i32.const 129
              local.get 3
              i32.sub
              call 41
              br_if 1 (;@4;)
              br 2 (;@3;)
            end
            local.get 3
            i32.load
            local.set 4
            i32.const 129
            local.set 3
            loop  ;; label = @5
              local.get 2
              local.get 3
              i32.add
              i32.const 30
              i32.add
              local.get 4
              i32.const 15
              i32.and
              local.tee 5
              i32.const 48
              i32.or
              local.get 5
              i32.const 55
              i32.add
              local.get 5
              i32.const 10
              i32.lt_u
              select
              i32.store8
              local.get 3
              i32.const 1
              i32.sub
              local.set 3
              local.get 4
              i32.const 15
              i32.gt_u
              local.get 4
              i32.const 4
              i32.shr_u
              local.set 4
              br_if 0 (;@5;)
            end
            local.get 2
            i32.const 16
            i32.add
            i32.const 1048988
            i32.const 2
            local.get 2
            local.get 3
            i32.add
            i32.const 31
            i32.add
            i32.const 129
            local.get 3
            i32.sub
            call 41
            i32.eqz
            br_if 1 (;@3;)
          end
          i32.const 1
          local.set 5
          br 2 (;@1;)
        end
        i32.const 1
        local.set 5
        local.get 2
        i32.load offset=16
        i32.const 1048982
        i32.const 2
        local.get 2
        i32.load offset=20
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
      end
      block  ;; label = @2
        local.get 1
        i32.load8_u offset=10
        i32.const 128
        i32.and
        i32.eqz
        if  ;; label = @3
          local.get 1
          i32.load
          i32.const 1048980
          i32.const 2
          local.get 1
          i32.load offset=4
          i32.load offset=12
          call_indirect (type 1)
          if  ;; label = @4
            br 3 (;@1;)
          end
          local.get 0
          local.get 1
          call 55
          br_if 2 (;@1;)
          local.get 1
          i32.load offset=4
          local.set 4
          local.get 1
          i32.load
          local.set 3
          br 1 (;@2;)
        end
        local.get 2
        i32.const 1
        i32.store8
        local.get 2
        i32.const 1048952
        i32.store offset=36
        local.get 2
        local.get 1
        i32.load offset=4
        local.tee 4
        i32.store offset=20
        local.get 2
        local.get 1
        i32.load
        local.tee 3
        i32.store offset=16
        local.get 2
        local.get 1
        i64.load offset=8 align=4
        i64.store offset=40 align=4
        local.get 2
        local.get 2
        i32.store offset=24
        local.get 2
        local.get 2
        i32.const 16
        i32.add
        i32.store offset=32
        local.get 0
        local.get 2
        i32.const 32
        i32.add
        call 55
        br_if 1 (;@1;)
        local.get 2
        i32.load offset=32
        i32.const 1048982
        i32.const 2
        local.get 2
        i32.load offset=36
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
      end
      local.get 3
      i32.const 1048640
      i32.const 1
      local.get 4
      i32.load offset=12
      call_indirect (type 1)
      local.set 5
    end
    local.get 2
    i32.const 160
    i32.add
    global.set 0
    local.get 5)
  (func (;55;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i64)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 2
    global.set 0
    local.get 0
    i32.const 8
    i32.add
    i32.load
    local.set 4
    local.get 0
    i32.const 4
    i32.add
    i32.load
    local.set 5
    local.get 1
    i32.load
    i32.const 1048684
    i32.const 1
    local.get 1
    i32.load offset=4
    i32.load offset=12
    call_indirect (type 1)
    local.set 3
    block  ;; label = @1
      local.get 4
      i32.eqz
      if  ;; label = @2
        local.get 3
        local.set 0
        br 1 (;@1;)
      end
      block (result i32)  ;; label = @2
        i32.const 1
        local.get 3
        br_if 0 (;@2;)
        drop
        local.get 1
        i32.load8_u offset=10
        i32.const 128
        i32.and
        if  ;; label = @3
          i32.const 1
          local.get 1
          i32.load
          local.tee 3
          i32.const 1057247
          i32.const 1
          local.get 1
          i32.load offset=4
          local.tee 0
          i32.load offset=12
          call_indirect (type 1)
          br_if 1 (;@2;)
          drop
          local.get 2
          i32.const 1
          i32.store8 offset=15
          local.get 2
          local.get 0
          i32.store offset=4
          local.get 2
          local.get 3
          i32.store
          local.get 2
          i32.const 1048952
          i32.store offset=20
          local.get 2
          local.get 1
          i64.load offset=8 align=4
          i64.store offset=24 align=4
          local.get 2
          local.get 2
          i32.const 15
          i32.add
          i32.store offset=8
          local.get 2
          local.get 2
          i32.store offset=16
          i32.const 1
          local.get 5
          local.get 2
          i32.const 16
          i32.add
          call 56
          br_if 1 (;@2;)
          drop
          local.get 2
          i32.load offset=16
          i32.const 1048982
          i32.const 2
          local.get 2
          i32.load offset=20
          i32.load offset=12
          call_indirect (type 1)
          br 1 (;@2;)
        end
        local.get 5
        local.get 1
        call 56
      end
      local.set 0
      local.get 4
      i32.const 1
      i32.eq
      br_if 0 (;@1;)
      local.get 5
      i32.const 1
      i32.add
      local.set 3
      local.get 4
      i32.const 1
      i32.sub
      local.set 4
      loop  ;; label = @2
        local.get 0
        i32.const 1
        i32.and
        local.set 5
        block (result i32)  ;; label = @3
          i32.const 1
          local.get 5
          br_if 0 (;@3;)
          drop
          block  ;; label = @4
            local.get 1
            i32.load8_u offset=10
            i32.const 128
            i32.and
            if  ;; label = @5
              local.get 1
              i64.load align=4
              local.set 6
              local.get 2
              i32.const 1
              i32.store8 offset=15
              local.get 2
              local.get 6
              i64.store align=4
              local.get 2
              i32.const 1048952
              i32.store offset=20
              local.get 2
              local.get 1
              i64.load offset=8 align=4
              i64.store offset=24 align=4
              local.get 2
              local.get 2
              i32.const 15
              i32.add
              i32.store offset=8
              local.get 2
              local.get 2
              i32.store offset=16
              local.get 3
              local.get 2
              i32.const 16
              i32.add
              call 56
              i32.eqz
              br_if 1 (;@4;)
              i32.const 1
              br 2 (;@3;)
            end
            i32.const 1
            local.get 1
            i32.load
            i32.const 1048980
            i32.const 2
            local.get 1
            i32.load offset=4
            i32.load offset=12
            call_indirect (type 1)
            br_if 1 (;@3;)
            drop
            local.get 3
            local.get 1
            call 56
            br 1 (;@3;)
          end
          local.get 2
          i32.load offset=16
          i32.const 1048982
          i32.const 2
          local.get 2
          i32.load offset=20
          i32.load offset=12
          call_indirect (type 1)
        end
        local.set 0
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        local.get 4
        i32.const 1
        i32.sub
        local.tee 4
        br_if 0 (;@2;)
      end
    end
    i32.const 1
    local.set 3
    local.get 0
    i32.eqz
    if  ;; label = @1
      local.get 1
      i32.load
      i32.const 1048987
      i32.const 1
      local.get 1
      i32.load offset=4
      i32.load offset=12
      call_indirect (type 1)
      local.set 3
    end
    local.get 2
    i32.const 32
    i32.add
    global.set 0
    local.get 3)
  (func (;56;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32)
    global.get 0
    i32.const 128
    i32.sub
    local.tee 4
    global.set 0
    block (result i32)  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load offset=8
        local.tee 2
        i32.const 33554432
        i32.and
        i32.eqz
        if  ;; label = @3
          local.get 2
          i32.const 67108864
          i32.and
          br_if 1 (;@2;)
          i32.const 3
          local.set 2
          local.get 0
          i32.load8_u
          local.tee 0
          local.set 3
          local.get 0
          i32.const 10
          i32.ge_u
          if  ;; label = @4
            local.get 4
            local.get 0
            local.get 0
            i32.const 100
            i32.div_u
            local.tee 3
            i32.const 100
            i32.mul
            i32.sub
            i32.const 255
            i32.and
            i32.const 1
            i32.shl
            local.tee 2
            i32.const 1048991
            i32.add
            i32.load8_u
            i32.store8 offset=2
            local.get 4
            local.get 2
            i32.const 1048990
            i32.add
            i32.load8_u
            i32.store8 offset=1
            i32.const 1
            local.set 2
          end
          local.get 3
          i32.eqz
          local.get 0
          i32.const 0
          i32.ne
          i32.and
          i32.eqz
          if  ;; label = @4
            local.get 2
            i32.const 1
            i32.sub
            local.tee 2
            local.get 4
            i32.add
            local.get 3
            i32.const 1
            i32.shl
            i32.const 254
            i32.and
            i32.const 1048991
            i32.add
            i32.load8_u
            i32.store8
          end
          local.get 1
          i32.const 1
          i32.const 0
          local.get 2
          local.get 4
          i32.add
          i32.const 3
          local.get 2
          i32.sub
          call 41
          br 2 (;@1;)
        end
        local.get 0
        i32.load8_u
        local.set 2
        i32.const 129
        local.set 0
        loop  ;; label = @3
          local.get 0
          local.get 4
          i32.add
          i32.const 2
          i32.sub
          local.get 2
          i32.const 15
          i32.and
          local.tee 3
          i32.const 48
          i32.or
          local.get 3
          i32.const 87
          i32.add
          local.get 3
          i32.const 10
          i32.lt_u
          select
          i32.store8
          local.get 2
          i32.const 255
          i32.and
          local.tee 3
          i32.const 4
          i32.shr_u
          local.set 2
          local.get 0
          i32.const 1
          i32.sub
          local.set 0
          local.get 3
          i32.const 15
          i32.gt_u
          br_if 0 (;@3;)
        end
        local.get 1
        i32.const 1048988
        i32.const 2
        local.get 0
        local.get 4
        i32.add
        i32.const 1
        i32.sub
        i32.const 129
        local.get 0
        i32.sub
        call 41
        br 1 (;@1;)
      end
      local.get 0
      i32.load8_u
      local.set 2
      i32.const 129
      local.set 0
      loop  ;; label = @2
        local.get 0
        local.get 4
        i32.add
        i32.const 2
        i32.sub
        local.get 2
        i32.const 15
        i32.and
        local.tee 3
        i32.const 48
        i32.or
        local.get 3
        i32.const 55
        i32.add
        local.get 3
        i32.const 10
        i32.lt_u
        select
        i32.store8
        local.get 2
        i32.const 255
        i32.and
        local.tee 3
        i32.const 4
        i32.shr_u
        local.set 2
        local.get 0
        i32.const 1
        i32.sub
        local.set 0
        local.get 3
        i32.const 15
        i32.gt_u
        br_if 0 (;@2;)
      end
      local.get 1
      i32.const 1048988
      i32.const 2
      local.get 0
      local.get 4
      i32.add
      i32.const 1
      i32.sub
      i32.const 129
      local.get 0
      i32.sub
      call 41
    end
    local.get 4
    i32.const 128
    i32.add
    global.set 0)
  (func (;57;) (type 3) (param i32 i32)
    (local i32 i32 i32 i64)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    i32.const 32
    i32.add
    local.tee 3
    i32.const 8
    i32.add
    local.tee 4
    i32.const 0
    i32.store8
    local.get 2
    i64.const 0
    i64.store offset=32
    local.get 3
    call 2
    local.get 2
    i32.const 8
    i32.add
    local.get 4
    i64.load
    local.tee 5
    i64.store
    local.get 2
    local.get 2
    i64.load offset=32
    i64.store
    block  ;; label = @1
      local.get 5
      i32.wrap_i64
      i32.const 255
      i32.and
      i32.eqz
      if  ;; label = @2
        local.get 2
        local.get 0
        local.get 1
        call 13
        br 1 (;@1;)
      end
      local.get 4
      i32.const 0
      i32.store8
      local.get 2
      i64.const 0
      i64.store offset=32
      local.get 2
      i32.const 32
      i32.add
      call 2
      local.get 2
      i32.const 16
      i32.add
      local.tee 3
      i32.const 8
      i32.add
      local.get 4
      i64.load
      i64.store
      local.get 2
      local.get 2
      i64.load offset=32
      i64.store offset=16
      local.get 3
      local.get 0
      local.get 1
      call 13
      local.get 2
      i32.const 0
      i32.store8 offset=8
      local.get 2
      i64.load offset=16
      local.set 5
      local.get 2
      local.get 2
      i64.load
      i64.store offset=16
      local.get 2
      local.get 5
      i64.store
      local.get 3
      call 7
    end
    local.get 2
    local.get 0
    local.get 1
    call 12
    local.get 1
    i32.const 1
    i32.store8 offset=8
    local.get 0
    i32.const 1
    i32.store8 offset=8
    local.get 0
    i64.load
    local.set 5
    local.get 0
    local.get 2
    i64.load
    i64.store
    local.get 2
    i32.const 1
    i32.store8 offset=8
    local.get 2
    local.get 5
    i64.store
    local.get 2
    call 7
    local.get 2
    i32.const 48
    i32.add
    global.set 0)
  (func (;58;) (type 2) (param i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i64)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 3
    global.set 0
    local.get 0
    call 59
    block  ;; label = @1
      local.get 0
      i32.load offset=84
      i32.eqz
      if  ;; label = @2
        br 1 (;@1;)
      end
      local.get 0
      local.get 0
      i32.load offset=80
      call 57
      i32.const 1
      local.set 5
      block (result i32)  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              i32.load offset=84
              local.tee 2
              i32.const 1
              i32.le_u
              br_if 0 (;@5;)
              local.get 0
              i32.const 16
              i32.add
              local.tee 1
              local.get 0
              i32.load offset=80
              i32.const 16
              i32.add
              call 57
              local.get 3
              i32.const 16
              i32.add
              local.tee 5
              local.get 0
              call 60
              local.get 0
              call 7
              local.get 0
              i32.const 8
              i32.add
              local.get 5
              i32.const 8
              i32.add
              local.tee 4
              i64.load
              i64.store
              local.get 0
              local.get 3
              i64.load offset=16
              i64.store
              local.get 5
              local.get 1
              call 60
              local.get 1
              call 7
              local.get 1
              i32.const 8
              i32.add
              local.get 4
              i64.load
              i64.store
              local.get 1
              local.get 3
              i64.load offset=16
              i64.store
              local.get 0
              call 59
              i32.const 3
              local.set 5
              local.get 0
              i32.load offset=84
              local.tee 2
              i32.const 3
              i32.lt_u
              if  ;; label = @6
                i32.const 2
                local.set 7
                br 5 (;@1;)
              end
              local.get 0
              local.get 0
              i32.load offset=80
              i32.const 32
              i32.add
              call 57
              i32.const 4
              local.set 7
              local.get 0
              i32.load offset=84
              local.tee 2
              i32.const 4
              i32.lt_u
              br_if 0 (;@5;)
              local.get 1
              local.get 0
              i32.load offset=80
              i32.const 48
              i32.add
              call 57
              local.get 3
              i32.const 16
              i32.add
              local.tee 4
              local.get 0
              call 60
              local.get 0
              call 7
              local.get 0
              i32.const 8
              i32.add
              local.tee 8
              local.get 4
              i32.const 8
              i32.add
              local.tee 6
              local.tee 5
              i64.load
              i64.store
              local.get 0
              local.get 3
              i64.load offset=16
              i64.store
              local.get 4
              local.get 1
              call 60
              local.get 1
              call 7
              local.get 1
              i32.const 8
              i32.add
              local.tee 9
              local.get 5
              i64.load
              i64.store
              local.get 1
              local.get 3
              i64.load offset=16
              i64.store
              local.get 0
              call 59
              i32.const 5
              local.set 5
              local.get 0
              i32.load offset=84
              local.tee 2
              i32.const 5
              i32.lt_u
              br_if 4 (;@1;)
              local.get 0
              local.get 0
              i32.load offset=80
              i32.const -64
              i32.sub
              call 57
              i32.const 6
              local.set 7
              local.get 0
              i32.load offset=84
              local.tee 2
              i32.const 6
              i32.lt_u
              br_if 0 (;@5;)
              local.get 1
              local.get 0
              i32.load offset=80
              i32.const 80
              i32.add
              call 57
              local.get 4
              local.get 0
              call 60
              local.get 0
              call 7
              local.get 8
              local.get 6
              i64.load
              i64.store
              local.get 0
              local.get 3
              i64.load offset=16
              i64.store
              local.get 4
              local.get 1
              call 60
              local.get 1
              call 7
              local.get 9
              local.get 6
              i64.load
              i64.store
              local.get 1
              local.get 3
              i64.load offset=16
              i64.store
              local.get 0
              call 59
              i32.const 7
              local.set 5
              local.get 0
              i32.load offset=84
              local.tee 2
              i32.const 7
              i32.lt_u
              br_if 4 (;@1;)
              local.get 0
              local.get 0
              i32.load offset=80
              i32.const 96
              i32.add
              call 57
              local.get 0
              i32.load offset=84
              local.tee 2
              i32.const 8
              i32.lt_u
              br_if 0 (;@5;)
              local.get 1
              local.get 0
              i32.load offset=80
              i32.const 112
              i32.add
              call 57
              local.get 4
              local.get 0
              call 60
              local.get 0
              call 7
              local.get 8
              local.get 6
              i64.load
              i64.store
              local.get 0
              local.get 3
              i64.load offset=16
              i64.store
              local.get 4
              local.get 1
              call 60
              local.get 1
              call 7
              local.get 9
              local.get 6
              i64.load
              i64.store
              local.get 1
              local.get 3
              i64.load offset=16
              i64.store
              local.get 0
              call 59
              i32.const 8
              local.get 0
              i32.load offset=52
              local.tee 4
              i32.eqz
              br_if 3 (;@2;)
              drop
              local.get 4
              i32.const 4
              i32.add
              local.set 8
              local.get 0
              i32.const 32
              i32.add
              local.set 5
              i32.const 128
              local.set 7
              i32.const 8
              local.set 2
              loop  ;; label = @6
                local.get 0
                i32.load offset=84
                local.tee 6
                local.get 2
                i32.le_u
                br_if 2 (;@4;)
                local.get 0
                local.get 0
                i32.load offset=80
                local.get 7
                i32.add
                call 57
                local.get 3
                i32.const 16
                i32.add
                local.tee 6
                local.get 0
                call 60
                local.get 0
                call 7
                local.get 0
                i32.const 8
                i32.add
                local.get 6
                i32.const 8
                i32.add
                local.tee 6
                i64.load
                i64.store
                local.get 0
                local.get 3
                i64.load offset=16
                i64.store
                block  ;; label = @7
                  local.get 0
                  i32.load8_u offset=40
                  i32.eqz
                  if  ;; label = @8
                    local.get 5
                    local.get 0
                    local.get 1
                    call 13
                    br 1 (;@7;)
                  end
                  local.get 6
                  i32.const 0
                  i32.store8
                  local.get 3
                  i64.const 0
                  i64.store offset=16
                  local.get 3
                  i32.const 16
                  i32.add
                  call 2
                  local.get 3
                  i32.const 8
                  i32.add
                  local.get 6
                  i64.load
                  i64.store
                  local.get 3
                  local.get 3
                  i64.load offset=16
                  i64.store
                  local.get 3
                  local.get 0
                  local.get 1
                  call 13
                  local.get 0
                  i32.const 0
                  i32.store8 offset=40
                  local.get 0
                  i64.load offset=32
                  local.set 10
                  local.get 0
                  local.get 3
                  i64.load
                  i64.store offset=32
                  local.get 3
                  local.get 10
                  i64.store
                  local.get 3
                  call 7
                end
                local.get 5
                local.get 0
                local.get 1
                call 12
                local.get 0
                i32.const 1
                i32.store8 offset=40
                local.get 0
                i32.const 1
                i32.store8 offset=24
                local.get 0
                i32.const 1
                i32.store8 offset=8
                local.get 0
                local.get 5
                call 57
                local.get 5
                local.get 1
                call 57
                local.get 1
                local.get 5
                call 57
                local.get 7
                i32.const 32
                i32.add
                local.set 7
                local.get 2
                i32.const 2
                i32.add
                local.set 2
                local.get 4
                i32.const 1
                i32.sub
                local.tee 4
                br_if 0 (;@6;)
              end
              br 2 (;@3;)
            end
            local.get 5
            local.get 2
            i32.const 1055772
            call 27
            unreachable
          end
          local.get 2
          local.get 6
          i32.const 1055788
          call 27
          unreachable
        end
        local.get 8
        i32.const 1
        i32.shl
      end
      local.set 5
      block  ;; label = @2
        local.get 5
        local.get 0
        i32.load offset=84
        local.tee 2
        i32.ge_u
        br_if 0 (;@2;)
        local.get 0
        local.get 0
        i32.load offset=80
        local.get 5
        i32.const 4
        i32.shl
        i32.add
        call 57
        block  ;; label = @3
          local.get 0
          i32.load offset=84
          local.tee 7
          local.get 5
          i32.const 1
          i32.or
          local.tee 2
          i32.le_u
          br_if 0 (;@3;)
          local.get 1
          local.get 0
          i32.load offset=80
          local.get 2
          i32.const 4
          i32.shl
          i32.add
          call 57
          local.get 3
          i32.const 16
          i32.add
          local.tee 4
          local.get 0
          call 60
          local.get 0
          call 7
          local.get 0
          i32.const 8
          i32.add
          local.get 4
          i32.const 8
          i32.add
          local.tee 2
          i64.load
          i64.store
          local.get 0
          local.get 3
          i64.load offset=16
          i64.store
          local.get 4
          local.get 1
          call 60
          local.get 1
          call 7
          local.get 1
          i32.const 8
          i32.add
          local.get 2
          i64.load
          i64.store
          local.get 1
          local.get 3
          i64.load offset=16
          i64.store
          local.get 0
          call 59
          local.get 0
          i32.load offset=84
          local.tee 2
          local.get 5
          i32.const 2
          i32.add
          local.tee 4
          i32.le_u
          if  ;; label = @4
            local.get 4
            local.set 5
            br 2 (;@2;)
          end
          local.get 0
          local.get 0
          i32.load offset=80
          local.get 4
          i32.const 4
          i32.shl
          i32.add
          call 57
          local.get 0
          i32.load offset=84
          local.tee 7
          local.get 4
          i32.const 1
          i32.or
          local.tee 2
          i32.le_u
          br_if 0 (;@3;)
          local.get 1
          local.get 0
          i32.load offset=80
          local.get 2
          i32.const 4
          i32.shl
          i32.add
          call 57
          local.get 3
          i32.const 16
          i32.add
          local.tee 4
          local.get 0
          call 60
          local.get 0
          call 7
          local.get 0
          i32.const 8
          i32.add
          local.get 4
          i32.const 8
          i32.add
          local.tee 2
          i64.load
          i64.store
          local.get 0
          local.get 3
          i64.load offset=16
          i64.store
          local.get 4
          local.get 1
          call 60
          local.get 1
          call 7
          local.get 1
          i32.const 8
          i32.add
          local.get 2
          i64.load
          i64.store
          local.get 1
          local.get 3
          i64.load offset=16
          i64.store
          local.get 0
          call 59
          local.get 0
          i32.load offset=84
          local.tee 2
          local.get 5
          i32.const 4
          i32.add
          local.tee 4
          i32.le_u
          if  ;; label = @4
            local.get 4
            local.set 5
            br 2 (;@2;)
          end
          local.get 0
          local.get 0
          i32.load offset=80
          local.get 4
          i32.const 4
          i32.shl
          i32.add
          call 57
          local.get 0
          i32.load offset=84
          local.tee 7
          local.get 4
          i32.const 1
          i32.or
          local.tee 2
          i32.le_u
          br_if 0 (;@3;)
          local.get 1
          local.get 0
          i32.load offset=80
          local.get 2
          i32.const 4
          i32.shl
          i32.add
          call 57
          local.get 3
          i32.const 16
          i32.add
          local.tee 4
          local.get 0
          call 60
          local.get 0
          call 7
          local.get 0
          i32.const 8
          i32.add
          local.tee 8
          local.get 4
          i32.const 8
          i32.add
          local.tee 6
          local.tee 2
          i64.load
          i64.store
          local.get 0
          local.get 3
          i64.load offset=16
          i64.store
          local.get 4
          local.get 1
          call 60
          local.get 1
          call 7
          local.get 1
          i32.const 8
          i32.add
          local.tee 9
          local.get 2
          i64.load
          i64.store
          local.get 1
          local.get 3
          i64.load offset=16
          i64.store
          local.get 0
          call 59
          local.get 0
          i32.load offset=84
          local.tee 2
          local.get 5
          i32.const 6
          i32.add
          local.tee 5
          i32.le_u
          br_if 1 (;@2;)
          local.get 0
          local.get 0
          i32.load offset=80
          local.get 5
          i32.const 4
          i32.shl
          i32.add
          call 57
          local.get 0
          i32.load offset=84
          local.tee 7
          local.get 5
          i32.const 1
          i32.or
          local.tee 2
          i32.le_u
          br_if 0 (;@3;)
          local.get 1
          local.get 0
          i32.load offset=80
          local.get 2
          i32.const 4
          i32.shl
          i32.add
          call 57
          local.get 4
          local.get 0
          call 60
          local.get 0
          call 7
          local.get 8
          local.get 6
          i64.load
          i64.store
          local.get 0
          local.get 3
          i64.load offset=16
          i64.store
          local.get 4
          local.get 1
          call 60
          local.get 1
          call 7
          local.get 9
          local.get 6
          i64.load
          i64.store
          local.get 1
          local.get 3
          i64.load offset=16
          i64.store
          local.get 0
          call 59
          local.get 3
          i32.const 32
          i32.add
          global.set 0
          return
        end
        local.get 2
        local.get 7
        i32.const 1055772
        call 27
        unreachable
      end
      local.get 5
      local.get 2
      i32.const 1055756
      call 27
      unreachable
    end
    local.get 7
    local.get 2
    i32.const 1055756
    call 27
    unreachable)
  (func (;59;) (type 2) (param i32)
    (local i32 i32 i32 i32 i32 i64)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 1
    global.set 0
    local.get 0
    i32.const 16
    i32.add
    local.set 2
    local.get 0
    i32.const 32
    i32.add
    local.set 3
    block  ;; label = @1
      local.get 0
      i32.load8_u offset=40
      i32.eqz
      if  ;; label = @2
        local.get 3
        local.get 0
        local.get 2
        call 13
        br 1 (;@1;)
      end
      local.get 1
      i32.const 16
      i32.add
      local.tee 4
      i32.const 8
      i32.add
      local.tee 5
      i32.const 0
      i32.store8
      local.get 1
      i64.const 0
      i64.store offset=16
      local.get 4
      call 2
      local.get 1
      i32.const 8
      i32.add
      local.get 5
      i64.load
      i64.store
      local.get 1
      local.get 1
      i64.load offset=16
      i64.store
      local.get 1
      local.get 0
      local.get 2
      call 13
      local.get 0
      i32.const 0
      i32.store8 offset=40
      local.get 0
      i64.load offset=32
      local.set 6
      local.get 0
      local.get 1
      i64.load
      i64.store offset=32
      local.get 1
      local.get 6
      i64.store
      local.get 1
      call 7
    end
    local.get 3
    local.get 0
    local.get 2
    call 12
    local.get 0
    i32.const 1
    i32.store8 offset=40
    local.get 0
    i32.const 1
    i32.store8 offset=24
    local.get 0
    i32.const 1
    i32.store8 offset=8
    local.get 0
    local.get 3
    call 57
    local.get 2
    local.get 3
    call 57
    local.get 1
    i32.const 32
    i32.add
    global.set 0)
  (func (;60;) (type 3) (param i32 i32)
    (local i32 i32 i32 i32 i64)
    global.get 0
    i32.const -64
    i32.add
    local.tee 2
    global.set 0
    local.get 2
    i32.const 48
    i32.add
    local.tee 4
    i32.const 8
    i32.add
    local.tee 3
    i32.const 0
    i32.store8
    local.get 2
    i64.const 0
    i64.store offset=48
    local.get 4
    call 2
    local.get 2
    i32.const 8
    i32.add
    local.tee 5
    local.get 3
    i64.load
    i64.store
    local.get 2
    local.get 2
    i64.load offset=48
    i64.store
    local.get 3
    i32.const 0
    i32.store8
    local.get 2
    i64.const 0
    i64.store offset=48
    local.get 4
    call 2
    local.get 2
    i32.const 24
    i32.add
    local.get 3
    i64.load
    i64.store
    local.get 2
    local.get 2
    i64.load offset=48
    i64.store offset=16
    block  ;; label = @1
      local.get 5
      i32.load8_u
      i32.eqz
      if  ;; label = @2
        local.get 2
        local.get 1
        local.get 1
        call 15
        br 1 (;@1;)
      end
      local.get 3
      i32.const 0
      i32.store8
      local.get 2
      i64.const 0
      i64.store offset=48
      local.get 2
      i32.const 48
      i32.add
      call 2
      local.get 2
      i32.const 32
      i32.add
      local.tee 4
      i32.const 8
      i32.add
      local.get 3
      i64.load
      i64.store
      local.get 2
      local.get 2
      i64.load offset=48
      i64.store offset=32
      local.get 4
      local.get 1
      local.get 1
      call 15
      local.get 2
      i32.const 0
      i32.store8 offset=8
      local.get 2
      i64.load offset=32
      local.set 6
      local.get 2
      local.get 2
      i64.load
      i64.store offset=32
      local.get 2
      local.get 6
      i64.store
      local.get 4
      call 7
    end
    local.get 2
    local.get 1
    local.get 1
    call 16
    local.get 1
    i32.const 1
    i32.store8 offset=8
    local.get 2
    i32.const 1
    i32.store8 offset=8
    block  ;; label = @1
      local.get 2
      i32.load8_u offset=24
      i32.eqz
      if  ;; label = @2
        local.get 2
        i32.const 16
        i32.add
        local.get 2
        local.get 2
        call 15
        br 1 (;@1;)
      end
      local.get 2
      i32.const 48
      i32.add
      local.tee 3
      i32.const 8
      i32.add
      local.tee 4
      i32.const 0
      i32.store8
      local.get 2
      i64.const 0
      i64.store offset=48
      local.get 3
      call 2
      local.get 2
      i32.const 32
      i32.add
      local.tee 3
      i32.const 8
      i32.add
      local.get 4
      i64.load
      i64.store
      local.get 2
      local.get 2
      i64.load offset=48
      i64.store offset=32
      local.get 3
      local.get 2
      local.get 2
      call 15
      local.get 2
      i32.const 0
      i32.store8 offset=24
      local.get 2
      i64.load offset=32
      local.set 6
      local.get 2
      local.get 2
      i64.load offset=16
      i64.store offset=32
      local.get 2
      local.get 6
      i64.store offset=16
      local.get 3
      call 7
    end
    local.get 2
    i32.const 16
    i32.add
    local.tee 3
    local.get 2
    local.get 2
    call 16
    local.get 3
    i32.const 8
    i32.add
    local.tee 4
    i32.const 1
    i32.store8
    local.get 2
    i32.const 1
    i32.store8 offset=8
    local.get 3
    local.get 1
    call 34
    local.get 0
    i32.const 8
    i32.add
    local.get 4
    i64.load
    i64.store
    local.get 0
    local.get 2
    i64.load offset=16
    i64.store
    local.get 2
    call 7
    local.get 2
    i32.const -64
    i32.sub
    global.set 0)
  (func (;61;) (type 12) (param i32 i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64)
    global.get 0
    i32.const 464
    i32.sub
    local.tee 5
    global.set 0
    i32.const 1057800
    i32.const 1057800
    i32.load
    local.tee 7
    i32.const 1
    i32.add
    i32.store
    local.get 5
    local.get 1
    i32.store offset=32
    local.get 5
    local.get 0
    i32.store offset=28
    local.get 5
    local.get 2
    i32.store offset=36
    block  ;; label = @1
      block  ;; label = @2
        local.get 7
        i32.const 0
        i32.ge_s
        if  ;; label = @3
          i32.const 1057836
          i32.load8_u
          i32.eqz
          if  ;; label = @4
            i32.const 1057836
            i32.const 1
            i32.store8
            i32.const 1057832
            i32.const 1057832
            i32.load
            i32.const 1
            i32.add
            i32.store
            i32.const 1057796
            i32.load
            local.tee 7
            i32.const 0
            i32.ge_s
            br_if 2 (;@2;)
            local.get 5
            i32.const 1
            i32.store offset=76
            local.get 5
            i32.const 1057740
            i32.store offset=72
            local.get 5
            i64.const 0
            i64.store offset=84 align=4
            local.get 5
            local.get 5
            i32.const 460
            i32.add
            local.tee 0
            i32.store offset=80
            local.get 5
            i32.const 48
            i32.add
            local.get 0
            local.get 5
            i32.const 72
            i32.add
            call 62
            local.get 5
            i32.load8_u offset=48
            local.get 5
            i32.load offset=52
            call 63
            br 3 (;@1;)
          end
          local.get 5
          i32.const 8
          i32.add
          local.get 0
          local.get 1
          i32.load offset=24
          call_indirect (type 3)
          local.get 5
          local.get 5
          i32.load offset=12
          i32.const 0
          local.get 5
          i32.load offset=8
          local.tee 0
          select
          i32.store offset=44
          local.get 5
          local.get 0
          i32.const 1
          local.get 0
          select
          i32.store offset=40
          local.get 5
          i32.const 3
          i32.store offset=76
          local.get 5
          i32.const 1057484
          i32.store offset=72
          local.get 5
          i64.const 2
          i64.store offset=84 align=4
          local.get 5
          local.get 5
          i32.const 40
          i32.add
          i64.extend_i32_u
          i64.const 21474836480
          i64.or
          i64.store offset=56
          local.get 5
          local.get 5
          i32.const 36
          i32.add
          i64.extend_i32_u
          i64.const 30064771072
          i64.or
          i64.store offset=48
          local.get 5
          local.get 5
          i32.const 48
          i32.add
          i32.store offset=80
          local.get 5
          i32.const -64
          i32.sub
          local.get 5
          i32.const 460
          i32.add
          local.get 5
          i32.const 72
          i32.add
          call 62
          local.get 5
          i32.load8_u offset=64
          local.get 5
          i32.load offset=68
          call 63
          br 2 (;@1;)
        end
        local.get 5
        i32.const 3
        i32.store offset=76
        local.get 5
        i32.const 1057408
        i32.store offset=72
        local.get 5
        i64.const 2
        i64.store offset=84 align=4
        local.get 5
        local.get 5
        i32.const 28
        i32.add
        i64.extend_i32_u
        i64.const 12884901888
        i64.or
        i64.store offset=56
        local.get 5
        local.get 5
        i32.const 36
        i32.add
        i64.extend_i32_u
        i64.const 30064771072
        i64.or
        i64.store offset=48
        local.get 5
        local.get 5
        i32.const 48
        i32.add
        i32.store offset=80
        local.get 5
        i32.const -64
        i32.sub
        local.get 5
        i32.const 460
        i32.add
        local.get 5
        i32.const 72
        i32.add
        call 62
        local.get 5
        i32.load8_u offset=64
        local.get 5
        i32.load offset=68
        call 63
        br 1 (;@1;)
      end
      i32.const 1057796
      local.get 7
      i32.const 1
      i32.add
      i32.store
      local.get 5
      i32.const 16
      i32.add
      local.get 0
      local.get 1
      i32.load offset=20
      call_indirect (type 3)
      local.get 5
      i32.load offset=20
      local.set 13
      local.get 5
      i32.load offset=16
      local.set 7
      i32.const 3
      local.set 0
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 4
              br_if 0 (;@5;)
              i32.const 1
              local.set 0
              i32.const 1057832
              i32.load
              i32.const 1
              i32.gt_u
              br_if 0 (;@5;)
              i32.const 1057793
              i32.load8_u
              i32.const 1
              i32.sub
              local.tee 0
              i32.const 255
              i32.and
              i32.const 3
              i32.lt_u
              br_if 0 (;@5;)
              local.get 5
              i32.const 0
              i32.store8 offset=86
              local.get 5
              i32.const 1056298
              i64.load align=1
              i64.store offset=78 align=2
              local.get 5
              i32.const 1056292
              i64.load align=1
              i64.store offset=72
              i32.const 16843008
              local.get 5
              i32.load offset=72
              local.tee 0
              i32.sub
              local.get 0
              i32.or
              i32.const 16843008
              local.get 5
              i32.load offset=76
              local.tee 0
              i32.sub
              local.get 0
              i32.or
              i32.and
              i32.const -2139062144
              i32.and
              i32.const -2139062144
              i32.eq
              i32.const 3
              i32.shl
              local.tee 0
              local.get 5
              i32.const 72
              i32.add
              i32.add
              local.set 4
              i32.const 0
              local.set 1
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    loop  ;; label = @9
                      local.get 1
                      local.get 4
                      i32.add
                      i32.load8_u
                      if  ;; label = @10
                        local.get 0
                        local.get 1
                        i32.const 1
                        i32.add
                        local.tee 1
                        i32.xor
                        i32.const 15
                        i32.ne
                        br_if 1 (;@9;)
                        br 2 (;@8;)
                      end
                    end
                    local.get 0
                    local.get 1
                    i32.add
                    i32.const 14
                    i32.ne
                    br_if 0 (;@8;)
                    i32.const 0
                    local.set 0
                    i32.const 1057756
                    i32.load
                    i32.const -1
                    i32.eq
                    if  ;; label = @9
                      global.get 0
                      i32.const 16
                      i32.sub
                      local.tee 1
                      global.set 0
                      block  ;; label = @10
                        local.get 1
                        i32.const 12
                        i32.add
                        local.get 1
                        i32.const 8
                        i32.add
                        call 22
                        i32.const 65535
                        i32.and
                        i32.eqz
                        if  ;; label = @11
                          local.get 1
                          i32.load offset=12
                          local.tee 4
                          i32.eqz
                          if  ;; label = @12
                            i32.const 1058340
                            local.set 4
                            br 2 (;@10;)
                          end
                          block  ;; label = @12
                            block  ;; label = @13
                              local.get 4
                              i32.const 1
                              i32.add
                              local.tee 4
                              i32.eqz
                              br_if 0 (;@13;)
                              local.get 1
                              i32.load offset=8
                              call 97
                              local.tee 6
                              i32.eqz
                              br_if 0 (;@13;)
                              local.get 4
                              i32.const 4
                              call 99
                              local.tee 4
                              br_if 1 (;@12;)
                              local.get 6
                              call 98
                            end
                            i32.const 70
                            call 105
                            unreachable
                          end
                          local.get 4
                          local.get 6
                          call 21
                          i32.const 65535
                          i32.and
                          i32.eqz
                          br_if 1 (;@10;)
                          local.get 6
                          call 98
                          local.get 4
                          call 98
                        end
                        i32.const 71
                        call 105
                        unreachable
                      end
                      i32.const 1057756
                      local.get 4
                      i32.store
                      local.get 1
                      i32.const 16
                      i32.add
                      global.set 0
                    end
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          local.get 5
                          i32.const 72
                          i32.add
                          local.tee 8
                          local.tee 1
                          i32.const 3
                          i32.and
                          i32.eqz
                          br_if 0 (;@11;)
                          local.get 1
                          i32.load8_u
                          local.tee 4
                          i32.eqz
                          br_if 2 (;@9;)
                          local.get 4
                          i32.const 61
                          i32.eq
                          br_if 2 (;@9;)
                          local.get 1
                          i32.const 1
                          i32.add
                          local.tee 4
                          i32.const 3
                          i32.and
                          i32.eqz
                          if  ;; label = @12
                            local.get 4
                            local.set 1
                            br 1 (;@11;)
                          end
                          local.get 4
                          i32.load8_u
                          local.tee 6
                          i32.eqz
                          br_if 1 (;@10;)
                          local.get 6
                          i32.const 61
                          i32.eq
                          br_if 1 (;@10;)
                          local.get 1
                          i32.const 2
                          i32.add
                          local.tee 4
                          i32.const 3
                          i32.and
                          i32.eqz
                          if  ;; label = @12
                            local.get 4
                            local.set 1
                            br 1 (;@11;)
                          end
                          local.get 4
                          i32.load8_u
                          local.tee 6
                          i32.eqz
                          br_if 1 (;@10;)
                          local.get 6
                          i32.const 61
                          i32.eq
                          br_if 1 (;@10;)
                          local.get 1
                          i32.const 3
                          i32.add
                          local.tee 4
                          i32.const 3
                          i32.and
                          i32.eqz
                          if  ;; label = @12
                            local.get 4
                            local.set 1
                            br 1 (;@11;)
                          end
                          local.get 4
                          i32.load8_u
                          local.tee 6
                          i32.eqz
                          br_if 1 (;@10;)
                          local.get 6
                          i32.const 61
                          i32.eq
                          br_if 1 (;@10;)
                          local.get 1
                          i32.const 4
                          i32.add
                          local.set 1
                        end
                        block  ;; label = @11
                          i32.const 16843008
                          local.get 1
                          i32.load
                          local.tee 4
                          i32.sub
                          local.get 4
                          i32.or
                          i32.const -2139062144
                          i32.and
                          i32.const -2139062144
                          i32.ne
                          br_if 0 (;@11;)
                          loop  ;; label = @12
                            i32.const 16843008
                            local.get 4
                            i32.const 1027423549
                            i32.xor
                            local.tee 4
                            i32.sub
                            local.get 4
                            i32.or
                            i32.const -2139062144
                            i32.and
                            i32.const -2139062144
                            i32.ne
                            br_if 1 (;@11;)
                            local.get 1
                            i32.load offset=4
                            local.set 4
                            local.get 1
                            i32.const 4
                            i32.add
                            local.set 1
                            local.get 4
                            i32.const 16843008
                            local.get 4
                            i32.sub
                            i32.or
                            i32.const -2139062144
                            i32.and
                            i32.const -2139062144
                            i32.eq
                            br_if 0 (;@12;)
                          end
                        end
                        local.get 1
                        i32.const 1
                        i32.sub
                        local.set 4
                        loop  ;; label = @11
                          local.get 4
                          i32.const 1
                          i32.add
                          local.tee 4
                          i32.load8_u
                          local.tee 1
                          i32.eqz
                          br_if 1 (;@10;)
                          local.get 1
                          i32.const 61
                          i32.ne
                          br_if 0 (;@11;)
                        end
                      end
                      local.get 4
                      local.set 1
                    end
                    local.get 1
                    local.get 8
                    i32.ne
                    if  ;; label = @9
                      block  ;; label = @10
                        local.get 1
                        local.get 8
                        i32.sub
                        local.tee 9
                        local.get 8
                        i32.add
                        i32.load8_u
                        br_if 0 (;@10;)
                        i32.const 1057756
                        i32.load
                        local.tee 1
                        i32.eqz
                        br_if 0 (;@10;)
                        local.get 1
                        i32.load
                        local.tee 4
                        i32.eqz
                        br_if 0 (;@10;)
                        local.get 1
                        i32.const 4
                        i32.add
                        local.set 10
                        loop  ;; label = @11
                          block  ;; label = @12
                            block (result i32)  ;; label = @13
                              local.get 4
                              local.set 1
                              i32.const 0
                              local.get 9
                              i32.eqz
                              br_if 0 (;@13;)
                              drop
                              block  ;; label = @14
                                local.get 8
                                i32.load8_u
                                local.tee 6
                                i32.eqz
                                if  ;; label = @15
                                  i32.const 0
                                  local.set 6
                                  br 1 (;@14;)
                                end
                                local.get 8
                                i32.const 1
                                i32.add
                                local.set 11
                                local.get 9
                                i32.const 1
                                i32.sub
                                local.set 12
                                block  ;; label = @15
                                  loop  ;; label = @16
                                    local.get 1
                                    i32.load8_u
                                    local.tee 14
                                    local.get 6
                                    i32.ne
                                    br_if 1 (;@15;)
                                    local.get 14
                                    i32.eqz
                                    br_if 1 (;@15;)
                                    local.get 12
                                    i32.eqz
                                    br_if 1 (;@15;)
                                    local.get 12
                                    i32.const 1
                                    i32.sub
                                    local.set 12
                                    local.get 1
                                    i32.const 1
                                    i32.add
                                    local.set 1
                                    local.get 11
                                    i32.load8_u
                                    local.set 6
                                    local.get 11
                                    i32.const 1
                                    i32.add
                                    local.set 11
                                    local.get 6
                                    br_if 0 (;@16;)
                                  end
                                  i32.const 0
                                  local.set 6
                                end
                              end
                              local.get 6
                              local.get 1
                              i32.load8_u
                              i32.sub
                            end
                            i32.eqz
                            if  ;; label = @13
                              local.get 4
                              local.get 9
                              i32.add
                              local.tee 1
                              i32.load8_u
                              i32.const 61
                              i32.eq
                              br_if 1 (;@12;)
                            end
                            local.get 10
                            i32.load
                            local.set 4
                            local.get 10
                            i32.const 4
                            i32.add
                            local.set 10
                            local.get 4
                            br_if 1 (;@11;)
                            br 2 (;@10;)
                          end
                        end
                        local.get 1
                        i32.const 1
                        i32.add
                        local.set 0
                      end
                    end
                    local.get 0
                    br_if 1 (;@7;)
                  end
                  i32.const 2
                  local.set 0
                  i32.const 3
                  local.set 4
                  br 1 (;@6;)
                end
                local.get 0
                call 107
                local.tee 1
                i32.const 0
                i32.lt_s
                br_if 2 (;@4;)
                block (result i32)  ;; label = @7
                  block  ;; label = @8
                    local.get 1
                    i32.const 0
                    i32.ne
                    local.tee 8
                    if  ;; label = @9
                      i32.const 1057764
                      i32.load8_u
                      drop
                      global.get 0
                      i32.const 16
                      i32.sub
                      local.tee 4
                      global.set 0
                      block (result i32)  ;; label = @10
                        local.get 1
                        i32.eqz
                        if  ;; label = @11
                          local.get 4
                          i32.const 0
                          i32.store offset=12
                          local.get 4
                          i32.const 12
                          i32.add
                          local.get 1
                          call 102
                          local.set 6
                          i32.const 0
                          local.get 4
                          i32.load offset=12
                          local.get 6
                          select
                          br 1 (;@10;)
                        end
                        local.get 1
                        call 97
                      end
                      local.set 6
                      local.get 4
                      i32.const 16
                      i32.add
                      global.set 0
                      local.get 6
                      i32.eqz
                      br_if 7 (;@2;)
                      local.get 8
                      if  ;; label = @10
                        local.get 6
                        local.get 0
                        local.get 1
                        memory.copy
                      end
                      block  ;; label = @10
                        block  ;; label = @11
                          local.get 1
                          i32.const 1
                          i32.sub
                          br_table 0 (;@11;) 3 (;@8;) 3 (;@8;) 1 (;@10;) 3 (;@8;)
                        end
                        local.get 6
                        i32.load8_u
                        i32.const 48
                        i32.ne
                        br_if 2 (;@8;)
                        i32.const 3
                        local.set 4
                        i32.const 2
                        br 3 (;@7;)
                      end
                      local.get 6
                      i32.load align=1
                      i32.const 1819047270
                      i32.ne
                      br_if 1 (;@8;)
                      i32.const 2
                      local.set 4
                      i32.const 1
                      br 2 (;@7;)
                    end
                    i32.const 1
                    local.set 4
                    local.get 1
                    if  ;; label = @9
                      i32.const 1
                      local.get 0
                      local.get 1
                      memory.copy
                    end
                    i32.const 0
                    local.set 0
                    br 2 (;@6;)
                  end
                  i32.const 1
                  local.set 4
                  i32.const 0
                end
                local.set 0
                local.get 6
                call 98
              end
              i32.const 1057793
              i32.const 1057793
              i32.load8_u
              local.tee 1
              local.get 4
              local.get 1
              select
              i32.store8
              local.get 1
              i32.eqz
              br_if 0 (;@5;)
              i32.const 3
              local.set 0
              local.get 1
              i32.const 3
              i32.gt_u
              br_if 0 (;@5;)
              i32.const 33619971
              local.get 1
              i32.const 3
              i32.shl
              i32.const 248
              i32.and
              i32.shr_u
              local.set 0
            end
            local.get 5
            local.get 2
            i32.store offset=40
            i32.const 12
            local.set 4
            local.get 5
            i32.const 72
            i32.add
            local.get 7
            local.get 13
            i32.const 12
            i32.add
            i32.load
            local.tee 6
            call_indirect (type 3)
            block  ;; label = @5
              block (result i32)  ;; label = @6
                local.get 5
                i64.load offset=72
                i64.const -5076933981314334344
                i64.eq
                if  ;; label = @7
                  local.get 7
                  local.set 2
                  i32.const 4
                  local.get 5
                  i64.load offset=80
                  i64.const 7199936582794304877
                  i64.eq
                  br_if 1 (;@6;)
                  drop
                end
                local.get 5
                i32.const 72
                i32.add
                local.get 7
                local.get 6
                call_indirect (type 3)
                i32.const 1057368
                local.set 2
                local.get 5
                i64.load offset=72
                i64.const -5190768330908619786
                i64.ne
                br_if 1 (;@5;)
                local.get 5
                i64.load offset=80
                i64.const 3353964679774260343
                i64.ne
                br_if 1 (;@5;)
                local.get 7
                i32.const 4
                i32.add
                local.set 2
                i32.const 8
              end
              local.get 7
              i32.add
              i32.load
              local.set 4
              local.get 2
              i32.load
              local.set 2
            end
            i32.const 1057794
            i32.load8_u
            local.set 1
            i32.const 1057794
            i32.const 1
            i32.store8
            local.get 5
            local.get 4
            i32.store offset=68
            local.get 5
            local.get 2
            i32.store offset=64
            local.get 5
            local.get 1
            i32.store8 offset=48
            local.get 1
            br_if 1 (;@3;)
            local.get 5
            i32.const 1057068
            i32.store offset=84
            local.get 5
            local.get 5
            i32.const 460
            i32.add
            i32.store offset=80
            local.get 5
            local.get 5
            i32.const -64
            i32.sub
            i32.store offset=76
            local.get 5
            local.get 5
            i32.const 40
            i32.add
            i32.store offset=72
            block  ;; label = @5
              block  ;; label = @6
                i32.const 1057824
                i64.load
                local.tee 15
                i64.const 0
                i64.ne
                if  ;; label = @7
                  i32.const 1057808
                  i64.load
                  local.get 15
                  i64.eq
                  br_if 1 (;@6;)
                end
                local.get 5
                i32.const 72
                i32.add
                i32.const 0
                local.get 1
                call 65
                br 1 (;@5;)
              end
              local.get 5
              i32.const 72
              i32.add
              i32.const 1056288
              i32.const 4
              call 65
            end
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 0
                    i32.const 255
                    i32.and
                    i32.const 1
                    i32.sub
                    br_table 1 (;@7;) 2 (;@6;) 3 (;@5;) 0 (;@8;)
                  end
                  local.get 5
                  i32.const 72
                  i32.add
                  local.get 5
                  i32.const 460
                  i32.add
                  i32.const 0
                  call 66
                  local.get 5
                  i32.load8_u offset=72
                  local.get 5
                  i32.load offset=76
                  call 63
                  br 2 (;@5;)
                end
                local.get 5
                i32.const 72
                i32.add
                local.get 5
                i32.const 460
                i32.add
                i32.const 1
                call 66
                local.get 5
                i32.load8_u offset=72
                local.get 5
                i32.load offset=76
                call 63
                br 1 (;@5;)
              end
              i32.const 1057748
              i32.load8_u
              i32.const 1057748
              i32.const 0
              i32.store8
              i32.eqz
              br_if 0 (;@5;)
              local.get 5
              i32.const 0
              i32.store offset=88
              local.get 5
              i32.const 1
              i32.store offset=76
              local.get 5
              i32.const 1057188
              i32.store offset=72
              local.get 5
              i64.const 4
              i64.store offset=80 align=4
              local.get 5
              i32.const 48
              i32.add
              local.get 5
              i32.const 460
              i32.add
              local.get 5
              i32.const 72
              i32.add
              call 62
              local.get 5
              i32.load8_u offset=48
              local.get 5
              i32.load offset=52
              call 63
            end
            i32.const 1057796
            i32.const 1057796
            i32.load
            i32.const 1
            i32.sub
            i32.store
            i32.const 1057794
            i32.const 0
            i32.store8
            i32.const 1057836
            i32.const 0
            i32.store8
            local.get 3
            i32.eqz
            if  ;; label = @5
              local.get 5
              i32.const 0
              i32.store offset=88
              local.get 5
              i32.const 1
              i32.store offset=76
              local.get 5
              i32.const 1057556
              i32.store offset=72
              local.get 5
              i64.const 4
              i64.store offset=80 align=4
              local.get 5
              i32.const 48
              i32.add
              local.get 5
              i32.const 460
              i32.add
              local.get 5
              i32.const 72
              i32.add
              call 62
              local.get 5
              i32.load8_u offset=48
              local.get 5
              i32.load offset=52
              call 63
              br 4 (;@1;)
            end
            unreachable
          end
          i32.const 1056124
          call 28
          unreachable
        end
        local.get 5
        i64.const 0
        i64.store offset=84 align=4
        local.get 5
        i64.const 17179869185
        i64.store offset=76 align=4
        local.get 5
        i32.const 1056792
        i32.store offset=72
        global.get 0
        i32.const 16
        i32.sub
        local.tee 1
        global.set 0
        local.get 1
        i32.const 1055833
        i32.store offset=12
        local.get 1
        local.get 5
        i32.const 48
        i32.add
        i32.store offset=8
        global.get 0
        i32.const 112
        i32.sub
        local.tee 0
        global.set 0
        local.get 0
        i32.const 1055836
        i32.store offset=12
        local.get 0
        local.get 1
        i32.const 8
        i32.add
        i32.store offset=8
        local.get 0
        i32.const 1055836
        i32.store offset=20
        local.get 0
        local.get 1
        i32.const 12
        i32.add
        i32.store offset=16
        local.get 0
        i32.const 2
        i32.store offset=28
        local.get 0
        i32.const 1048796
        i32.store offset=24
        block  ;; label = @3
          local.get 5
          i32.const 72
          i32.add
          local.tee 1
          i32.load
          if  ;; label = @4
            local.get 0
            i32.const 32
            i32.add
            local.tee 2
            i32.const 16
            i32.add
            local.get 1
            i32.const 16
            i32.add
            i64.load align=4
            i64.store
            local.get 2
            i32.const 8
            i32.add
            local.get 1
            i32.const 8
            i32.add
            i64.load align=4
            i64.store
            local.get 0
            local.get 1
            i64.load align=4
            i64.store offset=32
            local.get 0
            i32.const 4
            i32.store offset=92
            local.get 0
            i32.const 1048900
            i32.store offset=88
            local.get 0
            i64.const 4
            i64.store offset=100 align=4
            local.get 0
            local.get 0
            i32.const 16
            i32.add
            i64.extend_i32_u
            i64.const 12884901888
            i64.or
            i64.store offset=80
            local.get 0
            local.get 0
            i32.const 8
            i32.add
            i64.extend_i32_u
            i64.const 12884901888
            i64.or
            i64.store offset=72
            local.get 0
            local.get 2
            i64.extend_i32_u
            i64.const 17179869184
            i64.or
            i64.store offset=64
            br 1 (;@3;)
          end
          local.get 0
          i32.const 3
          i32.store offset=92
          local.get 0
          i32.const 1048848
          i32.store offset=88
          local.get 0
          i64.const 3
          i64.store offset=100 align=4
          local.get 0
          local.get 0
          i32.const 16
          i32.add
          i64.extend_i32_u
          i64.const 12884901888
          i64.or
          i64.store offset=72
          local.get 0
          local.get 0
          i32.const 8
          i32.add
          i64.extend_i32_u
          i64.const 12884901888
          i64.or
          i64.store offset=64
        end
        local.get 0
        local.get 0
        i32.const 24
        i32.add
        i64.extend_i32_u
        i64.const 21474836480
        i64.or
        i64.store offset=56
        local.get 0
        local.get 0
        i32.const 56
        i32.add
        i32.store offset=96
        local.get 0
        i32.const 88
        i32.add
        i32.const 1056844
        call 39
        unreachable
      end
      local.get 1
      call 29
      unreachable
    end
    unreachable)
  (func (;62;) (type 4) (param i32 i32 i32)
    (local i32 i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 3
    global.set 0
    local.get 2
    i32.load offset=4
    drop
    local.get 3
    i32.const 4
    i32.store8
    local.get 3
    local.get 1
    i32.store offset=8
    block  ;; label = @1
      block  ;; label = @2
        local.get 3
        i32.const 1056000
        local.get 2
        call 48
        if  ;; label = @3
          local.get 3
          i32.load8_u
          i32.const 4
          i32.ne
          br_if 1 (;@2;)
          local.get 3
          i32.const 0
          i32.store offset=28
          local.get 3
          i32.const 1
          i32.store offset=16
          local.get 3
          i32.const 1056524
          i32.store offset=12
          local.get 3
          i64.const 4
          i64.store offset=20 align=4
          local.get 3
          i32.const 12
          i32.add
          i32.const 1056532
          call 39
          unreachable
        end
        local.get 0
        i32.const 4
        i32.store8
        local.get 3
        i32.load offset=4
        local.set 0
        local.get 3
        i32.load8_u
        local.tee 1
        i32.const 3
        i32.ne
        local.get 1
        i32.const 4
        i32.le_u
        i32.and
        br_if 1 (;@1;)
        local.get 0
        i32.load
        local.set 1
        local.get 0
        i32.const 4
        i32.add
        i32.load
        local.tee 2
        i32.load
        local.tee 4
        if  ;; label = @3
          local.get 1
          local.get 4
          call_indirect (type 2)
        end
        local.get 2
        i32.load offset=4
        if  ;; label = @3
          local.get 1
          call 98
        end
        local.get 0
        call 98
        br 1 (;@1;)
      end
      local.get 0
      local.get 3
      i64.load
      i64.store align=4
    end
    local.get 3
    i32.const 48
    i32.add
    global.set 0)
  (func (;63;) (type 3) (param i32 i32)
    (local i32 i32)
    local.get 0
    i32.const 255
    i32.and
    local.tee 0
    i32.const 3
    i32.ne
    local.get 0
    i32.const 4
    i32.le_u
    i32.and
    i32.eqz
    if  ;; label = @1
      local.get 1
      i32.load
      local.set 0
      local.get 1
      i32.const 4
      i32.add
      i32.load
      local.tee 2
      i32.load
      local.tee 3
      if  ;; label = @2
        local.get 0
        local.get 3
        call_indirect (type 2)
      end
      local.get 2
      i32.load offset=4
      if  ;; label = @2
        local.get 0
        call 98
      end
      local.get 1
      call 98
    end)
  (func (;64;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 2
    global.set 0
    local.get 1
    i32.load offset=4
    local.set 3
    local.get 1
    i32.load
    local.get 0
    i32.load
    local.set 0
    local.get 2
    i32.const 3
    i32.store offset=4
    local.get 2
    i32.const 1055952
    i32.store
    local.get 2
    i64.const 3
    i64.store offset=12 align=4
    local.get 2
    local.get 0
    i64.extend_i32_u
    i64.const 21474836480
    i64.or
    i64.store offset=24
    local.get 2
    local.get 0
    i32.const 12
    i32.add
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=40
    local.get 2
    local.get 0
    i32.const 8
    i32.add
    i64.extend_i32_u
    i64.const 4294967296
    i64.or
    i64.store offset=32
    local.get 2
    local.get 2
    i32.const 24
    i32.add
    i32.store offset=8
    local.get 3
    local.get 2
    call 48
    local.get 2
    i32.const 48
    i32.add
    global.set 0)
  (func (;65;) (type 4) (param i32 i32 i32)
    (local i32 i32 i32 i64 i64 i64)
    global.get 0
    i32.const 624
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    local.get 2
    i32.const 9
    local.get 1
    select
    i32.store offset=4
    local.get 3
    local.get 1
    i32.const 1057196
    local.get 1
    select
    i32.store
    local.get 3
    i32.const 8
    i32.add
    local.tee 1
    i32.const 0
    i32.const 512
    memory.fill
    local.get 3
    i64.const 0
    i64.store offset=528
    local.get 3
    i32.const 512
    i32.store offset=524
    local.get 3
    local.get 1
    i32.store offset=520
    local.get 0
    i64.load32_u
    local.set 6
    local.get 0
    i64.load32_u offset=4
    local.set 7
    local.get 3
    i32.const 4
    i32.store offset=540
    local.get 3
    i32.const 1057248
    i32.store offset=536
    local.get 3
    i64.const 3
    i64.store offset=548 align=4
    local.get 3
    local.get 7
    i64.const 21474836480
    i64.or
    local.tee 7
    i64.store offset=576
    local.get 3
    local.get 6
    i64.const 30064771072
    i64.or
    local.tee 6
    i64.store offset=568
    local.get 3
    local.get 3
    i64.extend_i32_u
    i64.const 21474836480
    i64.or
    local.tee 8
    i64.store offset=560
    local.get 3
    local.get 3
    i32.const 560
    i32.add
    i32.store offset=544
    local.get 3
    i32.const 4
    i32.store8 offset=588
    local.get 3
    local.get 3
    i32.const 520
    i32.add
    i32.store offset=596
    local.get 3
    i32.const 588
    i32.add
    i32.const 1055976
    local.get 3
    i32.const 536
    i32.add
    call 48
    local.set 2
    local.get 3
    i32.load8_u offset=588
    local.set 1
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 2
              if  ;; label = @6
                local.get 1
                i32.const 4
                i32.ne
                br_if 1 (;@5;)
                local.get 3
                i32.const 0
                i32.store offset=616
                local.get 3
                i32.const 1
                i32.store offset=604
                local.get 3
                i32.const 1056524
                i32.store offset=600
                local.get 3
                i64.const 4
                i64.store offset=608 align=4
                local.get 3
                i32.const 600
                i32.add
                i32.const 1056532
                call 39
                unreachable
              end
              i32.const 23
              local.get 1
              i32.shr_u
              i32.const 1
              i32.and
              br_if 1 (;@4;)
              local.get 3
              i32.load offset=592
              local.tee 1
              i32.load
              local.set 2
              local.get 1
              i32.const 4
              i32.add
              i32.load
              local.tee 4
              i32.load
              local.tee 5
              if  ;; label = @6
                local.get 2
                local.get 5
                call_indirect (type 2)
              end
              local.get 4
              i32.load offset=4
              if  ;; label = @6
                local.get 2
                call 98
              end
              local.get 1
              call 98
              br 1 (;@4;)
            end
            local.get 3
            i32.load offset=588
            local.tee 1
            i32.const 255
            i32.and
            i32.const 4
            i32.ne
            br_if 1 (;@3;)
          end
          local.get 3
          i32.load offset=528
          local.tee 1
          i32.const 513
          i32.ge_u
          br_if 2 (;@1;)
          local.get 3
          i32.const 600
          i32.add
          local.get 0
          i32.load offset=8
          local.get 3
          i32.const 8
          i32.add
          local.get 1
          local.get 0
          i32.load offset=12
          i32.load offset=28
          call_indirect (type 5)
          local.get 3
          i32.load offset=604
          local.set 0
          local.get 3
          i32.load8_u offset=600
          local.tee 1
          i32.const 3
          i32.ne
          local.get 1
          i32.const 4
          i32.le_u
          i32.and
          br_if 1 (;@2;)
          local.get 0
          i32.load
          local.set 1
          local.get 0
          i32.const 4
          i32.add
          i32.load
          local.tee 2
          i32.load
          local.tee 4
          if  ;; label = @4
            local.get 1
            local.get 4
            call_indirect (type 2)
          end
          local.get 2
          i32.load offset=4
          if  ;; label = @4
            local.get 1
            call 98
          end
          local.get 0
          call 98
          br 1 (;@2;)
        end
        local.get 1
        i32.const 255
        i32.and
        i32.const 3
        i32.ge_u
        if  ;; label = @3
          local.get 3
          i32.load offset=592
          local.tee 1
          i32.load
          local.set 2
          local.get 1
          i32.const 4
          i32.add
          i32.load
          local.tee 4
          i32.load
          local.tee 5
          if  ;; label = @4
            local.get 2
            local.get 5
            call_indirect (type 2)
          end
          local.get 4
          i32.load offset=4
          if  ;; label = @4
            local.get 2
            call 98
          end
          local.get 1
          call 98
        end
        local.get 0
        i32.load offset=12
        i32.const 36
        i32.add
        i32.load
        local.set 1
        local.get 0
        i32.load offset=8
        local.set 0
        local.get 3
        i32.const 1057248
        i32.store offset=560
        local.get 3
        i64.const 3
        i64.store offset=572 align=4
        local.get 3
        local.get 7
        i64.store offset=616
        local.get 3
        local.get 6
        i64.store offset=608
        local.get 3
        local.get 8
        i64.store offset=600
        local.get 3
        local.get 3
        i32.const 600
        i32.add
        i32.store offset=568
        local.get 3
        i32.const 4
        i32.store offset=564
        local.get 3
        i32.const 536
        i32.add
        local.get 0
        local.get 3
        i32.const 560
        i32.add
        local.get 1
        call_indirect (type 4)
        local.get 3
        i32.load offset=540
        local.set 0
        local.get 3
        i32.load8_u offset=536
        local.tee 1
        i32.const 3
        i32.ne
        local.get 1
        i32.const 4
        i32.le_u
        i32.and
        br_if 0 (;@2;)
        local.get 0
        i32.load
        local.set 1
        local.get 0
        i32.const 4
        i32.add
        i32.load
        local.tee 2
        i32.load
        local.tee 4
        if  ;; label = @3
          local.get 1
          local.get 4
          call_indirect (type 2)
        end
        local.get 2
        i32.load offset=4
        if  ;; label = @3
          local.get 1
          call 98
        end
        local.get 0
        call 98
      end
      local.get 3
      i32.const 624
      i32.add
      global.set 0
      return
    end
    local.get 1
    i32.const 512
    i32.const 1057208
    call 32
    unreachable)
  (func (;66;) (type 4) (param i32 i32 i32)
    (local i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    i32.const 1
    i32.store offset=12
    local.get 3
    i32.const 1056308
    i32.store offset=8
    local.get 3
    i64.const 1
    i64.store offset=20 align=4
    local.get 3
    local.get 2
    i32.store8 offset=47
    local.get 3
    local.get 3
    i32.const 47
    i32.add
    i64.extend_i32_u
    i64.const 34359738368
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 0
    local.get 1
    local.get 3
    i32.const 8
    i32.add
    call 62
    local.get 3
    i32.const 48
    i32.add
    global.set 0)
  (func (;67;) (type 4) (param i32 i32 i32)
    (local i32 i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 3
    global.set 0
    block  ;; label = @1
      block (result i32)  ;; label = @2
        i32.const 0
        local.get 1
        local.get 1
        local.get 2
        i32.add
        local.tee 1
        i32.gt_u
        br_if 0 (;@2;)
        drop
        i32.const 0
        i32.const 8
        local.get 1
        local.get 0
        i32.load
        local.tee 2
        i32.const 1
        i32.shl
        local.tee 4
        local.get 1
        local.get 4
        i32.gt_u
        select
        local.tee 1
        local.get 1
        i32.const 8
        i32.le_u
        select
        local.tee 1
        i32.const 0
        i32.lt_s
        br_if 0 (;@2;)
        drop
        local.get 3
        local.get 2
        if (result i32)  ;; label = @3
          local.get 3
          local.get 2
          i32.store offset=28
          local.get 3
          local.get 0
          i32.load offset=4
          i32.store offset=20
          i32.const 1
        else
          i32.const 0
        end
        i32.store offset=24
        local.get 3
        i32.const 8
        i32.add
        local.set 4
        block (result i32)  ;; label = @3
          local.get 3
          i32.const 20
          i32.add
          local.tee 2
          i32.load offset=4
          if  ;; label = @4
            local.get 2
            i32.load offset=8
            if  ;; label = @5
              local.get 2
              i32.load
              local.get 1
              call 100
              br 2 (;@3;)
            end
          end
          i32.const 1057764
          i32.load8_u
          drop
          local.get 1
          call 97
        end
        local.set 2
        local.get 4
        local.get 1
        i32.store offset=8
        local.get 4
        local.get 2
        i32.const 1
        local.get 2
        select
        i32.store offset=4
        local.get 4
        local.get 2
        i32.eqz
        i32.store
        local.get 3
        i32.load offset=8
        i32.const 1
        i32.ne
        br_if 1 (;@1;)
        local.get 3
        i32.load offset=16
        local.set 0
        local.get 3
        i32.load offset=12
      end
      local.get 0
      i32.const 1055932
      call 38
      unreachable
    end
    local.get 3
    i32.load offset=12
    local.set 2
    local.get 0
    local.get 1
    i32.store
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 3
    i32.const 32
    i32.add
    global.set 0)
  (func (;68;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.load
    i32.load8_u
    i32.eqz
    if  ;; label = @1
      local.get 1
      i32.const 1049190
      i32.const 5
      call 44
      return
    end
    local.get 1
    i32.const 1049195
    i32.const 4
    call 44)
  (func (;69;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i64)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 6
    global.set 0
    i32.const 1057764
    i32.load8_u
    drop
    local.get 1
    i32.load offset=4
    local.set 8
    local.get 1
    i32.load
    local.set 7
    local.get 0
    i32.load8_u
    local.set 9
    i32.const 512
    local.set 1
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 512
          call 97
          local.tee 0
          if  ;; label = @4
            local.get 6
            local.get 0
            i32.store offset=8
            local.get 6
            i32.const 512
            i32.store offset=4
            loop  ;; label = @5
              block  ;; label = @6
                i32.const 1057752
                i32.load
                local.set 5
                block  ;; label = @7
                  local.get 0
                  i32.eqz
                  if  ;; label = @8
                    local.get 5
                    call 107
                    i32.const 1
                    i32.add
                    local.tee 3
                    call 97
                    local.tee 2
                    if  ;; label = @9
                      local.get 2
                      local.get 5
                      local.get 3
                      call 106
                      drop
                    end
                    local.get 2
                    br_if 1 (;@7;)
                    i32.const 1058336
                    i32.const 48
                    i32.store
                    i32.const 0
                    local.set 2
                    br 1 (;@7;)
                  end
                  local.get 1
                  local.get 5
                  call 107
                  i32.const 1
                  i32.add
                  i32.lt_u
                  if  ;; label = @8
                    i32.const 1058336
                    i32.const 68
                    i32.store
                    i32.const 0
                    local.set 2
                    br 1 (;@7;)
                  end
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 5
                      local.get 0
                      local.tee 2
                      i32.xor
                      i32.const 3
                      i32.and
                      if  ;; label = @10
                        local.get 5
                        i32.load8_u
                        local.set 4
                        br 1 (;@9;)
                      end
                      block  ;; label = @10
                        local.get 5
                        i32.const 3
                        i32.and
                        i32.eqz
                        if  ;; label = @11
                          local.get 5
                          local.set 3
                          br 1 (;@10;)
                        end
                        local.get 2
                        local.get 5
                        i32.load8_u
                        local.tee 3
                        i32.store8
                        local.get 3
                        i32.eqz
                        br_if 2 (;@8;)
                        local.get 2
                        i32.const 1
                        i32.add
                        local.set 4
                        local.get 5
                        i32.const 1
                        i32.add
                        local.tee 3
                        i32.const 3
                        i32.and
                        i32.eqz
                        if  ;; label = @11
                          local.get 4
                          local.set 2
                          br 1 (;@10;)
                        end
                        local.get 4
                        local.get 3
                        i32.load8_u
                        local.tee 3
                        i32.store8
                        local.get 3
                        i32.eqz
                        br_if 2 (;@8;)
                        local.get 2
                        i32.const 2
                        i32.add
                        local.set 4
                        local.get 5
                        i32.const 2
                        i32.add
                        local.tee 3
                        i32.const 3
                        i32.and
                        i32.eqz
                        if  ;; label = @11
                          local.get 4
                          local.set 2
                          br 1 (;@10;)
                        end
                        local.get 4
                        local.get 3
                        i32.load8_u
                        local.tee 3
                        i32.store8
                        local.get 3
                        i32.eqz
                        br_if 2 (;@8;)
                        local.get 2
                        i32.const 3
                        i32.add
                        local.set 4
                        local.get 5
                        i32.const 3
                        i32.add
                        local.tee 3
                        i32.const 3
                        i32.and
                        i32.eqz
                        if  ;; label = @11
                          local.get 4
                          local.set 2
                          br 1 (;@10;)
                        end
                        local.get 4
                        local.get 3
                        i32.load8_u
                        local.tee 3
                        i32.store8
                        local.get 3
                        i32.eqz
                        br_if 2 (;@8;)
                        local.get 2
                        i32.const 4
                        i32.add
                        local.set 2
                        local.get 5
                        i32.const 4
                        i32.add
                        local.set 3
                      end
                      i32.const 16843008
                      local.get 3
                      i32.load
                      local.tee 4
                      i32.sub
                      local.get 4
                      i32.or
                      i32.const -2139062144
                      i32.and
                      i32.const -2139062144
                      i32.ne
                      if  ;; label = @10
                        local.get 3
                        local.set 5
                        br 1 (;@9;)
                      end
                      loop  ;; label = @10
                        local.get 2
                        local.get 4
                        i32.store
                        local.get 2
                        i32.const 4
                        i32.add
                        local.set 2
                        local.get 3
                        i32.load offset=4
                        local.set 4
                        local.get 3
                        i32.const 4
                        i32.add
                        local.tee 5
                        local.set 3
                        local.get 4
                        i32.const 16843008
                        local.get 4
                        i32.sub
                        i32.or
                        i32.const -2139062144
                        i32.and
                        i32.const -2139062144
                        i32.eq
                        br_if 0 (;@10;)
                      end
                    end
                    local.get 2
                    local.get 4
                    i32.store8
                    local.get 4
                    i32.const 255
                    i32.and
                    i32.eqz
                    br_if 0 (;@8;)
                    local.get 5
                    i32.const 1
                    i32.add
                    local.set 3
                    local.get 2
                    local.set 4
                    loop  ;; label = @9
                      local.get 4
                      local.get 3
                      i32.load8_u
                      local.tee 2
                      i32.store8 offset=1
                      local.get 3
                      i32.const 1
                      i32.add
                      local.set 3
                      local.get 4
                      i32.const 1
                      i32.add
                      local.set 4
                      local.get 2
                      br_if 0 (;@9;)
                    end
                  end
                  local.get 0
                  local.set 2
                end
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 2
                    i32.eqz
                    if  ;; label = @9
                      i32.const 1058336
                      i32.load
                      local.tee 2
                      i32.const 68
                      i32.eq
                      br_if 2 (;@7;)
                      local.get 2
                      i64.extend_i32_u
                      i64.const 32
                      i64.shl
                      local.set 10
                      local.get 1
                      if  ;; label = @10
                        local.get 0
                        call 98
                      end
                      i32.const -2147483648
                      local.set 1
                      br 1 (;@8;)
                    end
                    local.get 6
                    local.get 0
                    call 107
                    local.tee 2
                    i32.store offset=12
                    local.get 1
                    local.get 2
                    i32.gt_u
                    if  ;; label = @9
                      block  ;; label = @10
                        local.get 2
                        i32.eqz
                        if  ;; label = @11
                          local.get 0
                          call 98
                          i32.const 1
                          local.set 1
                          br 1 (;@10;)
                        end
                        local.get 0
                        local.get 2
                        call 100
                        local.tee 1
                        i32.eqz
                        br_if 7 (;@3;)
                      end
                      local.get 6
                      local.get 1
                      i32.store offset=8
                      local.get 2
                      local.set 1
                    end
                    local.get 6
                    i64.load offset=8 align=4
                    local.set 10
                  end
                  block  ;; label = @8
                    local.get 1
                    i32.const -2147483648
                    i32.ne
                    br_if 0 (;@8;)
                    local.get 10
                    i64.const 255
                    i64.and
                    i64.const 3
                    i64.ne
                    br_if 0 (;@8;)
                    local.get 10
                    i64.const 32
                    i64.shr_u
                    i32.wrap_i64
                    local.tee 0
                    i32.load
                    local.set 2
                    local.get 0
                    i32.const 4
                    i32.add
                    i32.load
                    local.tee 5
                    i32.load
                    local.tee 3
                    if  ;; label = @9
                      local.get 2
                      local.get 3
                      call_indirect (type 2)
                    end
                    local.get 5
                    i32.load offset=4
                    if  ;; label = @9
                      local.get 2
                      call 98
                    end
                    local.get 0
                    call 98
                  end
                  local.get 7
                  i32.const 1056912
                  i32.const 17
                  local.get 8
                  i32.load offset=12
                  local.tee 0
                  call_indirect (type 1)
                  br_if 1 (;@6;)
                  local.get 9
                  i32.const 1
                  i32.and
                  i32.eqz
                  if  ;; label = @8
                    local.get 7
                    i32.const 1056929
                    i32.const 88
                    local.get 0
                    call_indirect (type 1)
                    br_if 2 (;@6;)
                  end
                  i32.const 0
                  local.set 0
                  local.get 1
                  i32.const -2147483648
                  i32.or
                  i32.const -2147483648
                  i32.eq
                  br_if 6 (;@1;)
                  br 5 (;@2;)
                end
                local.get 6
                local.get 1
                i32.store offset=12
                local.get 6
                i32.const 4
                i32.add
                local.get 1
                i32.const 1
                call 67
                local.get 6
                i32.load offset=8
                local.set 0
                local.get 6
                i32.load offset=4
                local.set 1
                br 1 (;@5;)
              end
            end
            i32.const 1
            local.set 0
            local.get 1
            i32.const -2147483648
            i32.or
            i32.const -2147483648
            i32.ne
            br_if 2 (;@2;)
            br 3 (;@1;)
          end
          i32.const 512
          call 29
          unreachable
        end
        local.get 2
        call 29
        unreachable
      end
      local.get 10
      i32.wrap_i64
      call 98
    end
    local.get 6
    i32.const 16
    i32.add
    global.set 0
    local.get 0)
  (func (;70;) (type 2) (param i32)
    (local i32 i32 i32)
    local.get 0
    i32.load offset=4
    local.set 1
    local.get 0
    i32.load8_u
    local.tee 0
    i32.const 3
    i32.ne
    local.get 0
    i32.const 4
    i32.le_u
    i32.and
    i32.eqz
    if  ;; label = @1
      local.get 1
      i32.load
      local.set 0
      local.get 1
      i32.const 4
      i32.add
      i32.load
      local.tee 2
      i32.load
      local.tee 3
      if  ;; label = @2
        local.get 0
        local.get 3
        call_indirect (type 2)
      end
      local.get 2
      i32.load offset=4
      if  ;; label = @2
        local.get 0
        call 98
      end
      local.get 1
      call 98
    end)
  (func (;71;) (type 1) (param i32 i32 i32) (result i32)
    (local i64 i64 i32 i32 i32 i32 i32)
    local.get 0
    i32.load offset=8
    local.tee 5
    i32.load offset=4
    local.tee 6
    i64.const 4294967295
    local.get 5
    i64.load offset=8
    local.tee 3
    local.get 3
    i64.const 4294967295
    i64.ge_u
    select
    i32.wrap_i64
    i32.sub
    local.tee 7
    i32.const 0
    local.get 6
    local.get 7
    i32.ge_u
    select
    local.tee 7
    local.get 2
    local.get 2
    local.get 7
    i32.gt_u
    select
    local.tee 8
    if  ;; label = @1
      local.get 5
      i32.load
      local.get 3
      local.get 6
      i64.extend_i32_u
      local.tee 4
      local.get 3
      local.get 4
      i64.lt_u
      select
      i32.wrap_i64
      i32.add
      local.get 1
      local.get 8
      memory.copy
    end
    local.get 5
    local.get 3
    local.get 8
    i64.extend_i32_u
    i64.add
    i64.store offset=8
    block  ;; label = @1
      local.get 2
      local.get 7
      i32.le_u
      br_if 0 (;@1;)
      i32.const 1056360
      i64.load
      local.tee 3
      i64.const 255
      i64.and
      i64.const 4
      i64.eq
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=4
      local.set 1
      local.get 0
      i32.load8_u
      local.tee 2
      i32.const 3
      i32.ne
      local.get 2
      i32.const 4
      i32.le_u
      i32.and
      i32.eqz
      if  ;; label = @2
        local.get 1
        i32.load
        local.set 2
        local.get 1
        i32.const 4
        i32.add
        i32.load
        local.tee 5
        i32.load
        local.tee 6
        if  ;; label = @3
          local.get 2
          local.get 6
          call_indirect (type 2)
        end
        local.get 5
        i32.load offset=4
        if  ;; label = @3
          local.get 2
          call 98
        end
        local.get 1
        call 98
      end
      local.get 0
      local.get 3
      i64.store align=4
      i32.const 1
      local.set 9
    end
    local.get 9)
  (func (;72;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i64 i64)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    i32.const 0
    i32.store offset=12
    block (result i32)  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.const 128
        i32.ge_u
        if  ;; label = @3
          local.get 1
          i32.const 2048
          i32.lt_u
          br_if 1 (;@2;)
          local.get 1
          i32.const 65536
          i32.ge_u
          if  ;; label = @4
            local.get 2
            local.get 1
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=15
            local.get 2
            local.get 1
            i32.const 18
            i32.shr_u
            i32.const 240
            i32.or
            i32.store8 offset=12
            local.get 2
            local.get 1
            i32.const 6
            i32.shr_u
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=14
            local.get 2
            local.get 1
            i32.const 12
            i32.shr_u
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=13
            i32.const 4
            br 3 (;@1;)
          end
          local.get 2
          local.get 1
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=14
          local.get 2
          local.get 1
          i32.const 12
          i32.shr_u
          i32.const 224
          i32.or
          i32.store8 offset=12
          local.get 2
          local.get 1
          i32.const 6
          i32.shr_u
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=13
          i32.const 3
          br 2 (;@1;)
        end
        local.get 2
        local.get 1
        i32.store8 offset=12
        i32.const 1
        br 1 (;@1;)
      end
      local.get 2
      local.get 1
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=13
      local.get 2
      local.get 1
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      i32.store8 offset=12
      i32.const 2
    end
    local.set 1
    local.get 0
    i32.load offset=8
    local.tee 3
    i32.load offset=4
    local.tee 5
    i64.const 4294967295
    local.get 3
    i64.load offset=8
    local.tee 8
    local.get 8
    i64.const 4294967295
    i64.ge_u
    select
    i32.wrap_i64
    i32.sub
    local.tee 4
    i32.const 0
    local.get 4
    local.get 5
    i32.le_u
    select
    local.tee 4
    local.get 1
    local.get 1
    local.get 4
    i32.gt_u
    select
    local.tee 6
    if  ;; label = @1
      local.get 3
      i32.load
      local.get 8
      local.get 5
      i64.extend_i32_u
      local.tee 9
      local.get 8
      local.get 9
      i64.lt_u
      select
      i32.wrap_i64
      i32.add
      local.get 2
      i32.const 12
      i32.add
      local.get 6
      memory.copy
    end
    local.get 3
    local.get 8
    local.get 6
    i64.extend_i32_u
    i64.add
    i64.store offset=8
    block  ;; label = @1
      local.get 1
      local.get 4
      i32.le_u
      br_if 0 (;@1;)
      i32.const 1056360
      i64.load
      local.tee 8
      i64.const 255
      i64.and
      i64.const 4
      i64.eq
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=4
      local.set 1
      local.get 0
      i32.load8_u
      local.tee 3
      i32.const 3
      i32.ne
      local.get 3
      i32.const 4
      i32.le_u
      i32.and
      i32.eqz
      if  ;; label = @2
        local.get 1
        i32.load
        local.set 3
        local.get 1
        i32.const 4
        i32.add
        i32.load
        local.tee 5
        i32.load
        local.tee 4
        if  ;; label = @3
          local.get 3
          local.get 4
          call_indirect (type 2)
        end
        local.get 5
        i32.load offset=4
        if  ;; label = @3
          local.get 3
          call 98
        end
        local.get 1
        call 98
      end
      local.get 0
      local.get 8
      i64.store align=4
      i32.const 1
      local.set 7
    end
    local.get 2
    i32.const 16
    i32.add
    global.set 0
    local.get 7)
  (func (;73;) (type 0) (param i32 i32) (result i32)
    local.get 1
    i32.load offset=4
    drop
    local.get 0
    i32.const 1055976
    local.get 1
    call 48)
  (func (;74;) (type 5) (param i32 i32 i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    local.get 1
    local.get 3
    i32.store offset=8
    local.get 1
    local.get 2
    i32.store offset=4
    block  ;; label = @1
      i32.const 2
      local.get 1
      i32.const 4
      i32.add
      i32.const 1
      local.get 1
      i32.const 12
      i32.add
      call 20
      local.tee 2
      if  ;; label = @2
        local.get 0
        local.get 2
        i32.const 65535
        i32.and
        i64.extend_i32_u
        i64.const 32
        i64.shl
        i64.store align=4
        br 1 (;@1;)
      end
      local.get 1
      i32.load offset=12
      local.set 2
      local.get 0
      i32.const 4
      i32.store8
      local.get 0
      local.get 2
      i32.store offset=4
    end
    local.get 1
    i32.const 16
    i32.add
    global.set 0)
  (func (;75;) (type 5) (param i32 i32 i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    block  ;; label = @1
      i32.const 2
      local.get 2
      local.get 3
      local.get 1
      i32.const 12
      i32.add
      call 20
      local.tee 2
      if  ;; label = @2
        local.get 0
        local.get 2
        i32.const 65535
        i32.and
        i64.extend_i32_u
        i64.const 32
        i64.shl
        i64.store align=4
        br 1 (;@1;)
      end
      local.get 1
      i32.load offset=12
      local.set 2
      local.get 0
      i32.const 4
      i32.store8
      local.get 0
      local.get 2
      i32.store offset=4
    end
    local.get 1
    i32.const 16
    i32.add
    global.set 0)
  (func (;76;) (type 6) (param i32) (result i32)
    unreachable)
  (func (;77;) (type 3) (param i32 i32)
    local.get 0
    i32.const 4
    i32.store8)
  (func (;78;) (type 5) (param i32 i32 i32 i32)
    (local i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    block  ;; label = @1
      block  ;; label = @2
        local.get 3
        if  ;; label = @3
          loop  ;; label = @4
            local.get 1
            local.get 3
            i32.store offset=8
            local.get 1
            local.get 2
            i32.store offset=4
            block  ;; label = @5
              i32.const 2
              local.get 1
              i32.const 4
              i32.add
              i32.const 1
              local.get 1
              i32.const 12
              i32.add
              call 20
              local.tee 4
              if  ;; label = @6
                local.get 4
                i32.const 65535
                i32.and
                local.tee 4
                i32.const 27
                i32.eq
                br_if 1 (;@5;)
                local.get 0
                local.get 4
                i64.extend_i32_u
                i64.const 32
                i64.shl
                i64.store align=4
                br 4 (;@2;)
              end
              local.get 1
              i32.load offset=12
              local.tee 4
              i32.eqz
              if  ;; label = @6
                local.get 0
                i32.const 1056360
                i64.load
                i64.store align=4
                br 4 (;@2;)
              end
              local.get 3
              local.get 4
              i32.lt_u
              br_if 4 (;@1;)
              local.get 2
              local.get 4
              i32.add
              local.set 2
              local.get 3
              local.get 4
              i32.sub
              local.set 3
            end
            local.get 3
            br_if 0 (;@4;)
          end
        end
        local.get 0
        i32.const 4
        i32.store8
      end
      local.get 1
      i32.const 16
      i32.add
      global.set 0
      return
    end
    local.get 4
    local.get 3
    i32.const 1056728
    call 43
    unreachable)
  (func (;79;) (type 5) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 4
    global.set 0
    block  ;; label = @1
      block  ;; label = @2
        local.get 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.const 4
        i32.add
        local.set 5
        local.get 3
        i32.const 3
        i32.shl
        local.set 6
        local.get 3
        i32.const 1
        i32.sub
        i32.const 536870911
        i32.and
        i32.const 1
        i32.add
        local.set 7
        i32.const 0
        local.set 1
        block  ;; label = @3
          loop  ;; label = @4
            local.get 5
            i32.load
            br_if 1 (;@3;)
            local.get 5
            i32.const 8
            i32.add
            local.set 5
            local.get 1
            i32.const 1
            i32.add
            local.set 1
            local.get 6
            i32.const 8
            i32.sub
            local.tee 6
            br_if 0 (;@4;)
          end
          local.get 7
          local.set 1
        end
        local.get 1
        local.get 3
        i32.le_u
        if  ;; label = @3
          local.get 1
          local.get 3
          i32.eq
          br_if 1 (;@2;)
          local.get 3
          local.get 1
          i32.sub
          local.set 6
          local.get 2
          local.get 1
          i32.const 3
          i32.shl
          i32.add
          local.set 3
          loop  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                i32.const 2
                local.get 3
                local.get 6
                local.get 4
                i32.const 8
                i32.add
                call 20
                local.tee 1
                if  ;; label = @7
                  local.get 1
                  i32.const 65535
                  i32.and
                  i32.const 27
                  i32.ne
                  br_if 1 (;@6;)
                  br 3 (;@4;)
                end
                local.get 4
                i32.load offset=8
                local.tee 5
                i32.eqz
                if  ;; label = @7
                  local.get 0
                  i32.const 1056360
                  i64.load
                  i64.store align=4
                  br 6 (;@1;)
                end
                local.get 3
                i32.const 4
                i32.add
                local.set 1
                local.get 6
                i32.const 3
                i32.shl
                local.set 8
                local.get 6
                i32.const 1
                i32.sub
                i32.const 536870911
                i32.and
                i32.const 1
                i32.add
                i32.const 0
                local.set 2
                loop  ;; label = @7
                  local.get 1
                  i32.load
                  local.tee 9
                  local.get 5
                  i32.gt_u
                  br_if 2 (;@5;)
                  local.get 1
                  i32.const 8
                  i32.add
                  local.set 1
                  local.get 2
                  i32.const 1
                  i32.add
                  local.set 2
                  local.get 5
                  local.get 9
                  i32.sub
                  local.set 5
                  local.get 8
                  i32.const 8
                  i32.sub
                  local.tee 8
                  br_if 0 (;@7;)
                end
                local.set 2
                br 1 (;@5;)
              end
              local.get 0
              local.get 1
              i32.const 65535
              i32.and
              i64.extend_i32_u
              i64.const 32
              i64.shl
              i64.store align=4
              br 4 (;@1;)
            end
            local.get 2
            local.get 6
            i32.le_u
            if  ;; label = @5
              local.get 2
              local.get 6
              i32.eq
              if  ;; label = @6
                local.get 5
                i32.eqz
                br_if 4 (;@2;)
                local.get 4
                i32.const 0
                i32.store offset=24
                local.get 4
                i32.const 1
                i32.store offset=12
                local.get 4
                i32.const 1056604
                i32.store offset=8
                local.get 4
                i64.const 4
                i64.store offset=16 align=4
                local.get 4
                i32.const 8
                i32.add
                i32.const 1056612
                call 39
                unreachable
              end
              local.get 5
              local.get 3
              local.get 2
              i32.const 3
              i32.shl
              i32.add
              local.tee 3
              i32.load offset=4
              local.tee 1
              i32.gt_u
              if  ;; label = @6
                local.get 4
                i32.const 0
                i32.store offset=24
                local.get 4
                i32.const 1
                i32.store offset=12
                local.get 4
                i32.const 1056664
                i32.store offset=8
                local.get 4
                i64.const 4
                i64.store offset=16 align=4
                local.get 4
                i32.const 8
                i32.add
                i32.const 1056712
                call 39
                unreachable
              end
              local.get 6
              local.get 2
              i32.sub
              local.set 6
              local.get 3
              local.get 1
              local.get 5
              i32.sub
              i32.store offset=4
              local.get 3
              local.get 3
              i32.load
              local.get 5
              i32.add
              i32.store
              br 1 (;@4;)
            end
          end
          local.get 2
          local.get 6
          i32.const 1056548
          call 43
          unreachable
        end
        local.get 1
        local.get 3
        i32.const 1056548
        call 43
        unreachable
      end
      local.get 0
      i32.const 4
      i32.store8
    end
    local.get 4
    i32.const 32
    i32.add
    global.set 0)
  (func (;80;) (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i64)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 4
    global.set 0
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        loop  ;; label = @3
          block  ;; label = @4
            local.get 4
            local.get 2
            i32.store offset=8
            local.get 4
            local.get 1
            i32.store offset=4
            block  ;; label = @5
              i32.const 2
              local.get 4
              i32.const 4
              i32.add
              i32.const 1
              local.get 4
              i32.const 12
              i32.add
              call 20
              local.tee 3
              if  ;; label = @6
                local.get 3
                i32.const 65535
                i32.and
                local.tee 3
                i32.const 27
                i32.eq
                br_if 1 (;@5;)
                local.get 3
                i64.extend_i32_u
                i64.const 32
                i64.shl
                local.set 6
                br 4 (;@2;)
              end
              local.get 4
              i32.load offset=12
              local.tee 3
              i32.eqz
              if  ;; label = @6
                i32.const 1056360
                i64.load
                local.set 6
                br 4 (;@2;)
              end
              local.get 2
              local.get 3
              i32.lt_u
              br_if 1 (;@4;)
              local.get 1
              local.get 3
              i32.add
              local.set 1
              local.get 2
              local.get 3
              i32.sub
              local.set 2
            end
            local.get 2
            br_if 1 (;@3;)
            br 3 (;@1;)
          end
        end
        local.get 3
        local.get 2
        i32.const 1056728
        call 43
        unreachable
      end
      local.get 6
      i64.const 255
      i64.and
      i64.const 4
      i64.eq
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=4
      local.set 1
      local.get 0
      i32.load8_u
      local.tee 2
      i32.const 3
      i32.ne
      local.get 2
      i32.const 4
      i32.le_u
      i32.and
      i32.eqz
      if  ;; label = @2
        local.get 1
        i32.load
        local.set 2
        local.get 1
        i32.const 4
        i32.add
        i32.load
        local.tee 3
        i32.load
        local.tee 5
        if  ;; label = @3
          local.get 2
          local.get 5
          call_indirect (type 2)
        end
        local.get 3
        i32.load offset=4
        if  ;; label = @3
          local.get 2
          call 98
        end
        local.get 1
        call 98
      end
      local.get 0
      local.get 6
      i64.store align=4
      i32.const 1
      local.set 5
    end
    local.get 4
    i32.const 16
    i32.add
    global.set 0
    local.get 5)
  (func (;81;) (type 0) (param i32 i32) (result i32)
    (local i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    i32.const 0
    i32.store offset=12
    local.get 0
    local.get 2
    i32.const 12
    i32.add
    block (result i32)  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.const 128
        i32.ge_u
        if  ;; label = @3
          local.get 1
          i32.const 2048
          i32.lt_u
          br_if 1 (;@2;)
          local.get 1
          i32.const 65536
          i32.ge_u
          if  ;; label = @4
            local.get 2
            local.get 1
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=15
            local.get 2
            local.get 1
            i32.const 18
            i32.shr_u
            i32.const 240
            i32.or
            i32.store8 offset=12
            local.get 2
            local.get 1
            i32.const 6
            i32.shr_u
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=14
            local.get 2
            local.get 1
            i32.const 12
            i32.shr_u
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=13
            i32.const 4
            br 3 (;@1;)
          end
          local.get 2
          local.get 1
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=14
          local.get 2
          local.get 1
          i32.const 12
          i32.shr_u
          i32.const 224
          i32.or
          i32.store8 offset=12
          local.get 2
          local.get 1
          i32.const 6
          i32.shr_u
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=13
          i32.const 3
          br 2 (;@1;)
        end
        local.get 2
        local.get 1
        i32.store8 offset=12
        i32.const 1
        br 1 (;@1;)
      end
      local.get 2
      local.get 1
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=13
      local.get 2
      local.get 1
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      i32.store8 offset=12
      i32.const 2
    end
    call 80
    local.get 2
    i32.const 16
    i32.add
    global.set 0)
  (func (;82;) (type 0) (param i32 i32) (result i32)
    local.get 1
    i32.load offset=4
    drop
    local.get 0
    i32.const 1056000
    local.get 1
    call 48)
  (func (;83;) (type 3) (param i32 i32)
    local.get 0
    i64.const 7199936582794304877
    i64.store offset=8
    local.get 0
    i64.const -5076933981314334344
    i64.store)
  (func (;84;) (type 0) (param i32 i32) (result i32)
    local.get 1
    i32.load
    local.get 0
    i32.load
    local.get 0
    i32.load offset=4
    local.get 1
    i32.load offset=4
    i32.load offset=12
    call_indirect (type 1))
  (func (;85;) (type 3) (param i32 i32)
    (local i32 i32)
    i32.const 1057764
    i32.load8_u
    drop
    local.get 1
    i32.load offset=4
    local.set 2
    local.get 1
    i32.load
    local.set 3
    i32.const 8
    call 97
    local.tee 1
    i32.eqz
    if  ;; label = @1
      i32.const 8
      call 29
      unreachable
    end
    local.get 1
    local.get 2
    i32.store offset=4
    local.get 1
    local.get 3
    i32.store
    local.get 0
    i32.const 1057296
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store)
  (func (;86;) (type 3) (param i32 i32)
    local.get 0
    i32.const 1057296
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store)
  (func (;87;) (type 3) (param i32 i32)
    local.get 0
    local.get 1
    i64.load align=4
    i64.store)
  (func (;88;) (type 2) (param i32)
    local.get 0
    i32.load
    i32.const -2147483648
    i32.or
    i32.const -2147483648
    i32.ne
    if  ;; label = @1
      local.get 0
      i32.load offset=4
      call 98
    end)
  (func (;89;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 2
    global.set 0
    block (result i32)  ;; label = @1
      local.get 0
      i32.load
      i32.const -2147483648
      i32.ne
      if  ;; label = @2
        local.get 1
        i32.load
        local.get 0
        i32.load offset=4
        local.get 0
        i32.load offset=8
        local.get 1
        i32.load offset=4
        i32.load offset=12
        call_indirect (type 1)
        br 1 (;@1;)
      end
      local.get 2
      i32.const 8
      i32.add
      local.tee 3
      i32.const 8
      i32.add
      local.get 0
      i32.load offset=12
      i32.load
      local.tee 0
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 3
      i32.const 16
      i32.add
      local.get 0
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 0
      i64.load align=4
      i64.store offset=8
      local.get 2
      i32.load offset=12
      drop
      local.get 1
      i32.load
      local.get 1
      i32.load offset=4
      local.get 3
      call 48
    end
    local.get 2
    i32.const 32
    i32.add
    global.set 0)
  (func (;90;) (type 3) (param i32 i32)
    (local i32 i32 i32 i64)
    global.get 0
    i32.const -64
    i32.add
    local.tee 2
    global.set 0
    local.get 1
    i32.load
    i32.const -2147483648
    i32.eq
    if  ;; label = @1
      local.get 1
      i32.load offset=12
      local.set 3
      local.get 2
      i32.const 0
      i32.store offset=36
      local.get 2
      i64.const 4294967296
      i64.store offset=28 align=4
      local.get 2
      i32.const 40
      i32.add
      local.tee 4
      i32.const 8
      i32.add
      local.get 3
      i32.load
      local.tee 3
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 4
      i32.const 16
      i32.add
      local.get 3
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 3
      i64.load align=4
      i64.store offset=40
      local.get 2
      i32.load offset=44
      drop
      local.get 2
      i32.const 28
      i32.add
      local.tee 3
      i32.const 1056024
      local.get 4
      call 48
      drop
      local.get 2
      i32.const 24
      i32.add
      local.get 3
      i32.const 8
      i32.add
      i32.load
      local.tee 3
      i32.store
      local.get 2
      local.get 2
      i64.load offset=28 align=4
      local.tee 5
      i64.store offset=16
      local.get 1
      i32.const 8
      i32.add
      local.get 3
      i32.store
      local.get 1
      local.get 5
      i64.store align=4
    end
    local.get 1
    i64.load align=4
    local.set 5
    local.get 1
    i64.const 4294967296
    i64.store align=4
    local.get 2
    i32.const 8
    i32.add
    local.tee 3
    local.get 1
    i32.const 8
    i32.add
    local.tee 1
    i32.load
    i32.store
    local.get 1
    i32.const 0
    i32.store
    i32.const 1057764
    i32.load8_u
    drop
    local.get 2
    local.get 5
    i64.store
    i32.const 12
    call 97
    local.tee 1
    i32.eqz
    if  ;; label = @1
      i32.const 12
      call 29
      unreachable
    end
    local.get 1
    local.get 2
    i64.load
    i64.store align=4
    local.get 1
    i32.const 8
    i32.add
    local.get 3
    i32.load
    i32.store
    local.get 0
    i32.const 1057280
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store
    local.get 2
    i32.const -64
    i32.sub
    global.set 0)
  (func (;91;) (type 3) (param i32 i32)
    (local i32 i32 i32 i64)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 2
    global.set 0
    local.get 1
    i32.load
    i32.const -2147483648
    i32.eq
    if  ;; label = @1
      local.get 1
      i32.load offset=12
      local.set 3
      local.get 2
      i32.const 0
      i32.store offset=20
      local.get 2
      i64.const 4294967296
      i64.store offset=12 align=4
      local.get 2
      i32.const 24
      i32.add
      local.tee 4
      i32.const 8
      i32.add
      local.get 3
      i32.load
      local.tee 3
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 4
      i32.const 16
      i32.add
      local.get 3
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 3
      i64.load align=4
      i64.store offset=24
      local.get 2
      i32.load offset=28
      drop
      local.get 2
      i32.const 12
      i32.add
      local.tee 3
      i32.const 1056024
      local.get 2
      i32.const 24
      i32.add
      call 48
      drop
      local.get 2
      i32.const 8
      i32.add
      local.get 3
      i32.const 8
      i32.add
      i32.load
      local.tee 3
      i32.store
      local.get 2
      local.get 2
      i64.load offset=12 align=4
      local.tee 5
      i64.store
      local.get 1
      i32.const 8
      i32.add
      local.get 3
      i32.store
      local.get 1
      local.get 5
      i64.store align=4
    end
    local.get 0
    i32.const 1057280
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store
    local.get 2
    i32.const 48
    i32.add
    global.set 0)
  (func (;92;) (type 3) (param i32 i32)
    local.get 0
    i32.const 0
    i32.store)
  (func (;93;) (type 3) (param i32 i32)
    local.get 0
    i64.const 3353964679774260343
    i64.store offset=8
    local.get 0
    i64.const -5190768330908619786
    i64.store)
  (func (;94;) (type 1) (param i32 i32 i32) (result i32)
    (local i32)
    local.get 0
    i32.load
    local.get 0
    i32.load offset=8
    local.tee 3
    i32.sub
    local.get 2
    i32.lt_u
    if  ;; label = @1
      local.get 0
      local.get 3
      local.get 2
      call 67
      local.get 0
      i32.load offset=8
      local.set 3
    end
    local.get 2
    if  ;; label = @1
      local.get 0
      i32.load offset=4
      local.get 3
      i32.add
      local.get 1
      local.get 2
      memory.copy
    end
    local.get 0
    local.get 2
    local.get 3
    i32.add
    i32.store offset=8
    i32.const 0)
  (func (;95;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32)
    local.get 0
    i32.load offset=8
    local.tee 3
    local.set 2
    block (result i32)  ;; label = @1
      i32.const 1
      local.get 1
      i32.const 128
      i32.lt_u
      br_if 0 (;@1;)
      drop
      i32.const 2
      local.get 1
      i32.const 2048
      i32.lt_u
      br_if 0 (;@1;)
      drop
      i32.const 3
      i32.const 4
      local.get 1
      i32.const 65536
      i32.lt_u
      select
    end
    local.tee 4
    local.get 0
    i32.load
    local.get 3
    i32.sub
    i32.gt_u
    if (result i32)  ;; label = @1
      local.get 0
      local.get 3
      local.get 4
      call 67
      local.get 0
      i32.load offset=8
    else
      local.get 2
    end
    local.get 0
    i32.load offset=4
    i32.add
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.const 128
        i32.ge_u
        if  ;; label = @3
          local.get 1
          i32.const 2048
          i32.lt_u
          br_if 1 (;@2;)
          local.get 1
          i32.const 65536
          i32.ge_u
          if  ;; label = @4
            local.get 2
            local.get 1
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=3
            local.get 2
            local.get 1
            i32.const 18
            i32.shr_u
            i32.const 240
            i32.or
            i32.store8
            local.get 2
            local.get 1
            i32.const 6
            i32.shr_u
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=2
            local.get 2
            local.get 1
            i32.const 12
            i32.shr_u
            i32.const 63
            i32.and
            i32.const 128
            i32.or
            i32.store8 offset=1
            br 3 (;@1;)
          end
          local.get 2
          local.get 1
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=2
          local.get 2
          local.get 1
          i32.const 12
          i32.shr_u
          i32.const 224
          i32.or
          i32.store8
          local.get 2
          local.get 1
          i32.const 6
          i32.shr_u
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=1
          br 2 (;@1;)
        end
        local.get 2
        local.get 1
        i32.store8
        br 1 (;@1;)
      end
      local.get 2
      local.get 1
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=1
      local.get 2
      local.get 1
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      i32.store8
    end
    local.get 0
    local.get 3
    local.get 4
    i32.add
    i32.store offset=8
    i32.const 0)
  (func (;96;) (type 0) (param i32 i32) (result i32)
    local.get 1
    i32.load offset=4
    drop
    local.get 0
    i32.const 1056024
    local.get 1
    call 48)
  (func (;97;) (type 6) (param i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 10
    global.set 0
    i32.const 1057864
    i32.load
    local.tee 7
    i32.eqz
    if  ;; label = @1
      i32.const 1058312
      i32.load
      local.tee 6
      i32.eqz
      if  ;; label = @2
        i32.const 1058324
        i64.const -1
        i64.store align=4
        i32.const 1058316
        i64.const 281474976776192
        i64.store align=4
        i32.const 1058312
        local.get 10
        i32.const 8
        i32.add
        i32.const -16
        i32.and
        i32.const 1431655768
        i32.xor
        local.tee 6
        i32.store
        i32.const 1058332
        i32.const 0
        i32.store
        i32.const 1058284
        i32.const 0
        i32.store
      end
      i32.const 1058288
      i32.const 1058352
      i32.store
      i32.const 1057856
      i32.const 1058352
      i32.store
      i32.const 1057876
      local.get 6
      i32.store
      i32.const 1057872
      i32.const -1
      i32.store
      i32.const 1058292
      i32.const 55760
      i32.store
      i32.const 1058276
      i32.const 55760
      i32.store
      i32.const 1058272
      i32.const 55760
      i32.store
      loop  ;; label = @2
        local.get 1
        i32.const 1057900
        i32.add
        local.get 1
        i32.const 1057888
        i32.add
        local.tee 2
        i32.store
        local.get 2
        local.get 1
        i32.const 1057880
        i32.add
        local.tee 4
        i32.store
        local.get 1
        i32.const 1057892
        i32.add
        local.get 4
        i32.store
        local.get 1
        i32.const 1057908
        i32.add
        local.get 1
        i32.const 1057896
        i32.add
        local.tee 4
        i32.store
        local.get 4
        local.get 2
        i32.store
        local.get 1
        i32.const 1057916
        i32.add
        local.get 1
        i32.const 1057904
        i32.add
        local.tee 2
        i32.store
        local.get 2
        local.get 4
        i32.store
        local.get 1
        i32.const 1057912
        i32.add
        local.get 2
        i32.store
        local.get 1
        i32.const 32
        i32.add
        local.tee 1
        i32.const 256
        i32.ne
        br_if 0 (;@2;)
      end
      i32.const 1114060
      i32.const 56
      i32.store
      i32.const 1057868
      i32.const 1058328
      i32.load
      i32.store
      i32.const 1057864
      i32.const 1058360
      i32.store
      i32.const 1057852
      i32.const 55696
      i32.store
      i32.const 1058364
      i32.const 55697
      i32.store
      i32.const 1058360
      local.set 7
    end
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            local.get 0
                            i32.const 236
                            i32.le_u
                            if  ;; label = @13
                              i32.const 1057840
                              i32.load
                              local.tee 5
                              i32.const 16
                              local.get 0
                              i32.const 19
                              i32.add
                              i32.const 496
                              i32.and
                              local.get 0
                              i32.const 11
                              i32.lt_u
                              select
                              local.tee 3
                              i32.const 3
                              i32.shr_u
                              local.tee 0
                              i32.shr_u
                              local.tee 1
                              i32.const 3
                              i32.and
                              if  ;; label = @14
                                block  ;; label = @15
                                  local.get 1
                                  i32.const 1
                                  i32.and
                                  local.get 0
                                  i32.or
                                  i32.const 1
                                  i32.xor
                                  local.tee 2
                                  i32.const 3
                                  i32.shl
                                  local.tee 0
                                  i32.const 1057880
                                  i32.add
                                  local.tee 1
                                  local.get 0
                                  i32.const 1057888
                                  i32.add
                                  i32.load
                                  local.tee 0
                                  i32.load offset=8
                                  local.tee 4
                                  i32.eq
                                  if  ;; label = @16
                                    i32.const 1057840
                                    local.get 5
                                    i32.const -2
                                    local.get 2
                                    i32.rotl
                                    i32.and
                                    i32.store
                                    br 1 (;@15;)
                                  end
                                  local.get 1
                                  local.get 4
                                  i32.store offset=8
                                  local.get 4
                                  local.get 1
                                  i32.store offset=12
                                end
                                local.get 0
                                i32.const 8
                                i32.add
                                local.set 1
                                local.get 0
                                local.get 2
                                i32.const 3
                                i32.shl
                                local.tee 2
                                i32.const 3
                                i32.or
                                i32.store offset=4
                                local.get 0
                                local.get 2
                                i32.add
                                local.tee 0
                                local.get 0
                                i32.load offset=4
                                i32.const 1
                                i32.or
                                i32.store offset=4
                                br 13 (;@1;)
                              end
                              i32.const 1057848
                              i32.load
                              local.tee 8
                              local.get 3
                              i32.ge_u
                              br_if 1 (;@12;)
                              local.get 1
                              if  ;; label = @14
                                block  ;; label = @15
                                  i32.const 2
                                  local.get 0
                                  i32.shl
                                  local.tee 2
                                  i32.const 0
                                  local.get 2
                                  i32.sub
                                  i32.or
                                  local.get 1
                                  local.get 0
                                  i32.shl
                                  i32.and
                                  i32.ctz
                                  local.tee 0
                                  i32.const 3
                                  i32.shl
                                  local.tee 2
                                  i32.const 1057880
                                  i32.add
                                  local.tee 1
                                  local.get 2
                                  i32.const 1057888
                                  i32.add
                                  i32.load
                                  local.tee 2
                                  i32.load offset=8
                                  local.tee 4
                                  i32.eq
                                  if  ;; label = @16
                                    i32.const 1057840
                                    local.get 5
                                    i32.const -2
                                    local.get 0
                                    i32.rotl
                                    i32.and
                                    local.tee 5
                                    i32.store
                                    br 1 (;@15;)
                                  end
                                  local.get 1
                                  local.get 4
                                  i32.store offset=8
                                  local.get 4
                                  local.get 1
                                  i32.store offset=12
                                end
                                local.get 2
                                local.get 3
                                i32.const 3
                                i32.or
                                i32.store offset=4
                                local.get 0
                                i32.const 3
                                i32.shl
                                local.tee 0
                                local.get 3
                                i32.sub
                                local.set 6
                                local.get 0
                                local.get 2
                                i32.add
                                local.get 6
                                i32.store
                                local.get 2
                                local.get 3
                                i32.add
                                local.tee 3
                                local.get 6
                                i32.const 1
                                i32.or
                                i32.store offset=4
                                local.get 8
                                if  ;; label = @15
                                  local.get 8
                                  i32.const -8
                                  i32.and
                                  i32.const 1057880
                                  i32.add
                                  local.set 0
                                  i32.const 1057860
                                  i32.load
                                  local.set 4
                                  block (result i32)  ;; label = @16
                                    i32.const 1
                                    local.get 8
                                    i32.const 3
                                    i32.shr_u
                                    i32.shl
                                    local.tee 1
                                    local.get 5
                                    i32.and
                                    i32.eqz
                                    if  ;; label = @17
                                      i32.const 1057840
                                      local.get 1
                                      local.get 5
                                      i32.or
                                      i32.store
                                      local.get 0
                                      br 1 (;@16;)
                                    end
                                    local.get 0
                                    i32.load offset=8
                                  end
                                  local.tee 1
                                  local.get 4
                                  i32.store offset=12
                                  local.get 0
                                  local.get 4
                                  i32.store offset=8
                                  local.get 4
                                  local.get 0
                                  i32.store offset=12
                                  local.get 4
                                  local.get 1
                                  i32.store offset=8
                                end
                                local.get 2
                                i32.const 8
                                i32.add
                                local.set 1
                                i32.const 1057860
                                local.get 3
                                i32.store
                                i32.const 1057848
                                local.get 6
                                i32.store
                                br 13 (;@1;)
                              end
                              i32.const 1057844
                              i32.load
                              local.tee 11
                              i32.eqz
                              br_if 1 (;@12;)
                              local.get 11
                              i32.ctz
                              i32.const 2
                              i32.shl
                              i32.const 1058144
                              i32.add
                              i32.load
                              local.tee 2
                              i32.load offset=4
                              i32.const -8
                              i32.and
                              local.get 3
                              i32.sub
                              local.set 6
                              local.get 2
                              local.set 0
                              loop  ;; label = @14
                                block  ;; label = @15
                                  local.get 0
                                  i32.load offset=16
                                  local.tee 1
                                  i32.eqz
                                  if  ;; label = @16
                                    local.get 0
                                    i32.load offset=20
                                    local.tee 1
                                    i32.eqz
                                    br_if 1 (;@15;)
                                  end
                                  local.get 1
                                  i32.load offset=4
                                  i32.const -8
                                  i32.and
                                  local.get 3
                                  i32.sub
                                  local.tee 4
                                  local.get 6
                                  i32.lt_u
                                  local.set 0
                                  local.get 4
                                  local.get 6
                                  local.get 0
                                  select
                                  local.set 6
                                  local.get 1
                                  local.get 2
                                  local.get 0
                                  select
                                  local.set 2
                                  local.get 1
                                  local.set 0
                                  br 1 (;@14;)
                                end
                              end
                              local.get 2
                              i32.load offset=24
                              local.set 9
                              local.get 2
                              local.get 2
                              i32.load offset=12
                              local.tee 1
                              i32.ne
                              if  ;; label = @14
                                local.get 2
                                i32.load offset=8
                                local.tee 0
                                local.get 1
                                i32.store offset=12
                                local.get 1
                                local.get 0
                                i32.store offset=8
                                br 12 (;@2;)
                              end
                              local.get 2
                              i32.load offset=20
                              local.tee 0
                              if (result i32)  ;; label = @14
                                local.get 2
                                i32.const 20
                                i32.add
                              else
                                local.get 2
                                i32.load offset=16
                                local.tee 0
                                i32.eqz
                                br_if 3 (;@11;)
                                local.get 2
                                i32.const 16
                                i32.add
                              end
                              local.set 4
                              loop  ;; label = @14
                                local.get 4
                                local.set 7
                                local.get 0
                                local.tee 1
                                i32.const 20
                                i32.add
                                local.set 4
                                local.get 1
                                i32.load offset=20
                                local.tee 0
                                br_if 0 (;@14;)
                                local.get 1
                                i32.const 16
                                i32.add
                                local.set 4
                                local.get 1
                                i32.load offset=16
                                local.tee 0
                                br_if 0 (;@14;)
                              end
                              local.get 7
                              i32.const 0
                              i32.store
                              br 11 (;@2;)
                            end
                            i32.const -1
                            local.set 3
                            local.get 0
                            i32.const -65
                            i32.gt_u
                            br_if 0 (;@12;)
                            local.get 0
                            i32.const 19
                            i32.add
                            local.tee 1
                            i32.const -16
                            i32.and
                            local.set 3
                            i32.const 1057844
                            i32.load
                            local.tee 8
                            i32.eqz
                            br_if 0 (;@12;)
                            i32.const 31
                            local.set 9
                            i32.const 0
                            local.get 3
                            i32.sub
                            local.set 6
                            local.get 0
                            i32.const 16777196
                            i32.le_u
                            if  ;; label = @13
                              local.get 3
                              i32.const 38
                              local.get 1
                              i32.const 8
                              i32.shr_u
                              i32.clz
                              local.tee 0
                              i32.sub
                              i32.shr_u
                              i32.const 1
                              i32.and
                              local.get 0
                              i32.const 1
                              i32.shl
                              i32.sub
                              i32.const 62
                              i32.add
                              local.set 9
                            end
                            block  ;; label = @13
                              block  ;; label = @14
                                block  ;; label = @15
                                  local.get 9
                                  i32.const 2
                                  i32.shl
                                  i32.const 1058144
                                  i32.add
                                  i32.load
                                  local.tee 0
                                  i32.eqz
                                  if  ;; label = @16
                                    i32.const 0
                                    local.set 1
                                    i32.const 0
                                    local.set 4
                                    br 1 (;@15;)
                                  end
                                  i32.const 0
                                  local.set 1
                                  local.get 3
                                  i32.const 25
                                  local.get 9
                                  i32.const 1
                                  i32.shr_u
                                  i32.sub
                                  i32.const 0
                                  local.get 9
                                  i32.const 31
                                  i32.ne
                                  select
                                  i32.shl
                                  local.set 2
                                  i32.const 0
                                  local.set 4
                                  loop  ;; label = @16
                                    block  ;; label = @17
                                      local.get 0
                                      i32.load offset=4
                                      i32.const -8
                                      i32.and
                                      local.get 3
                                      i32.sub
                                      local.tee 5
                                      local.get 6
                                      i32.ge_u
                                      br_if 0 (;@17;)
                                      local.get 0
                                      local.set 4
                                      local.get 5
                                      local.tee 6
                                      br_if 0 (;@17;)
                                      i32.const 0
                                      local.set 6
                                      local.get 0
                                      local.set 1
                                      br 3 (;@14;)
                                    end
                                    local.get 1
                                    local.get 0
                                    i32.load offset=20
                                    local.tee 5
                                    local.get 5
                                    local.get 0
                                    local.get 2
                                    i32.const 29
                                    i32.shr_u
                                    i32.const 4
                                    i32.and
                                    i32.add
                                    i32.const 16
                                    i32.add
                                    i32.load
                                    local.tee 0
                                    i32.eq
                                    select
                                    local.get 1
                                    local.get 5
                                    select
                                    local.set 1
                                    local.get 2
                                    i32.const 1
                                    i32.shl
                                    local.set 2
                                    local.get 0
                                    br_if 0 (;@16;)
                                  end
                                end
                                local.get 1
                                local.get 4
                                i32.or
                                i32.eqz
                                if  ;; label = @15
                                  i32.const 0
                                  local.set 4
                                  i32.const 2
                                  local.get 9
                                  i32.shl
                                  local.tee 0
                                  i32.const 0
                                  local.get 0
                                  i32.sub
                                  i32.or
                                  local.get 8
                                  i32.and
                                  local.tee 0
                                  i32.eqz
                                  br_if 3 (;@12;)
                                  local.get 0
                                  i32.ctz
                                  i32.const 2
                                  i32.shl
                                  i32.const 1058144
                                  i32.add
                                  i32.load
                                  local.set 1
                                end
                                local.get 1
                                i32.eqz
                                br_if 1 (;@13;)
                              end
                              loop  ;; label = @14
                                local.get 1
                                i32.load offset=4
                                i32.const -8
                                i32.and
                                local.get 3
                                i32.sub
                                local.tee 2
                                local.get 6
                                i32.lt_u
                                local.set 0
                                local.get 2
                                local.get 6
                                local.get 0
                                select
                                local.set 6
                                local.get 1
                                local.get 4
                                local.get 0
                                select
                                local.set 4
                                local.get 1
                                i32.load offset=16
                                local.tee 0
                                if (result i32)  ;; label = @15
                                  local.get 0
                                else
                                  local.get 1
                                  i32.load offset=20
                                end
                                local.tee 1
                                br_if 0 (;@14;)
                              end
                            end
                            local.get 4
                            i32.eqz
                            br_if 0 (;@12;)
                            local.get 6
                            i32.const 1057848
                            i32.load
                            local.get 3
                            i32.sub
                            i32.ge_u
                            br_if 0 (;@12;)
                            local.get 4
                            i32.load offset=24
                            local.set 7
                            local.get 4
                            local.get 4
                            i32.load offset=12
                            local.tee 1
                            i32.ne
                            if  ;; label = @13
                              local.get 4
                              i32.load offset=8
                              local.tee 0
                              local.get 1
                              i32.store offset=12
                              local.get 1
                              local.get 0
                              i32.store offset=8
                              br 10 (;@3;)
                            end
                            local.get 4
                            i32.load offset=20
                            local.tee 0
                            if (result i32)  ;; label = @13
                              local.get 4
                              i32.const 20
                              i32.add
                            else
                              local.get 4
                              i32.load offset=16
                              local.tee 0
                              i32.eqz
                              br_if 3 (;@10;)
                              local.get 4
                              i32.const 16
                              i32.add
                            end
                            local.set 2
                            loop  ;; label = @13
                              local.get 2
                              local.set 5
                              local.get 0
                              local.tee 1
                              i32.const 20
                              i32.add
                              local.set 2
                              local.get 1
                              i32.load offset=20
                              local.tee 0
                              br_if 0 (;@13;)
                              local.get 1
                              i32.const 16
                              i32.add
                              local.set 2
                              local.get 1
                              i32.load offset=16
                              local.tee 0
                              br_if 0 (;@13;)
                            end
                            local.get 5
                            i32.const 0
                            i32.store
                            br 9 (;@3;)
                          end
                          i32.const 1057848
                          i32.load
                          local.tee 4
                          local.get 3
                          i32.ge_u
                          if  ;; label = @12
                            i32.const 1057860
                            i32.load
                            local.set 1
                            block  ;; label = @13
                              local.get 4
                              local.get 3
                              i32.sub
                              local.tee 0
                              i32.const 16
                              i32.ge_u
                              if  ;; label = @14
                                local.get 1
                                local.get 3
                                i32.add
                                local.tee 2
                                local.get 0
                                i32.const 1
                                i32.or
                                i32.store offset=4
                                local.get 1
                                local.get 4
                                i32.add
                                local.get 0
                                i32.store
                                local.get 1
                                local.get 3
                                i32.const 3
                                i32.or
                                i32.store offset=4
                                br 1 (;@13;)
                              end
                              local.get 1
                              local.get 4
                              i32.const 3
                              i32.or
                              i32.store offset=4
                              local.get 1
                              local.get 4
                              i32.add
                              local.tee 0
                              local.get 0
                              i32.load offset=4
                              i32.const 1
                              i32.or
                              i32.store offset=4
                              i32.const 0
                              local.set 2
                              i32.const 0
                              local.set 0
                            end
                            i32.const 1057848
                            local.get 0
                            i32.store
                            i32.const 1057860
                            local.get 2
                            i32.store
                            local.get 1
                            i32.const 8
                            i32.add
                            local.set 1
                            br 11 (;@1;)
                          end
                          i32.const 1057852
                          i32.load
                          local.tee 2
                          local.get 3
                          i32.gt_u
                          if  ;; label = @12
                            local.get 3
                            local.get 7
                            i32.add
                            local.tee 0
                            local.get 2
                            local.get 3
                            i32.sub
                            local.tee 1
                            i32.const 1
                            i32.or
                            i32.store offset=4
                            i32.const 1057864
                            local.get 0
                            i32.store
                            i32.const 1057852
                            local.get 1
                            i32.store
                            local.get 7
                            local.get 3
                            i32.const 3
                            i32.or
                            i32.store offset=4
                            local.get 7
                            i32.const 8
                            i32.add
                            local.set 1
                            br 11 (;@1;)
                          end
                          i32.const 0
                          local.set 1
                          local.get 3
                          local.set 6
                          local.get 6
                          block (result i32)  ;; label = @12
                            i32.const 1058312
                            i32.load
                            if  ;; label = @13
                              i32.const 1058320
                              i32.load
                              br 1 (;@12;)
                            end
                            i32.const 1058324
                            i64.const -1
                            i64.store align=4
                            i32.const 1058316
                            i64.const 281474976776192
                            i64.store align=4
                            i32.const 1058312
                            local.get 10
                            i32.const 12
                            i32.add
                            i32.const -16
                            i32.and
                            i32.const 1431655768
                            i32.xor
                            i32.store
                            i32.const 1058332
                            i32.const 0
                            i32.store
                            i32.const 1058284
                            i32.const 0
                            i32.store
                            i32.const 65536
                          end
                          local.tee 0
                          local.get 6
                          i32.const 71
                          i32.add
                          local.tee 4
                          i32.add
                          local.tee 3
                          i32.const 0
                          local.get 0
                          i32.sub
                          local.tee 5
                          i32.and
                          local.tee 0
                          i32.ge_u
                          if  ;; label = @12
                            i32.const 1058336
                            i32.const 48
                            i32.store
                            br 11 (;@1;)
                          end
                          block  ;; label = @12
                            i32.const 1058280
                            i32.load
                            local.tee 8
                            i32.eqz
                            br_if 0 (;@12;)
                            i32.const 1058272
                            i32.load
                            local.tee 9
                            local.get 0
                            i32.add
                            local.set 1
                            local.get 1
                            local.get 8
                            i32.le_u
                            local.get 1
                            local.get 9
                            i32.gt_u
                            i32.and
                            br_if 0 (;@12;)
                            i32.const 0
                            local.set 1
                            i32.const 1058336
                            i32.const 48
                            i32.store
                            br 11 (;@1;)
                          end
                          i32.const 1058284
                          i32.load8_u
                          i32.const 4
                          i32.and
                          br_if 4 (;@7;)
                          block  ;; label = @12
                            block  ;; label = @13
                              local.get 7
                              if  ;; label = @14
                                i32.const 1058288
                                local.set 1
                                loop  ;; label = @15
                                  local.get 1
                                  i32.load
                                  local.tee 8
                                  local.get 7
                                  i32.le_u
                                  if  ;; label = @16
                                    local.get 8
                                    local.get 1
                                    i32.load offset=4
                                    i32.add
                                    local.get 7
                                    i32.gt_u
                                    br_if 3 (;@13;)
                                  end
                                  local.get 1
                                  i32.load offset=8
                                  local.tee 1
                                  br_if 0 (;@15;)
                                end
                              end
                              i32.const 0
                              call 104
                              local.tee 2
                              i32.const -1
                              i32.eq
                              br_if 5 (;@8;)
                              local.get 0
                              local.set 5
                              i32.const 1058316
                              i32.load
                              local.tee 1
                              i32.const 1
                              i32.sub
                              local.tee 3
                              local.get 2
                              i32.and
                              if  ;; label = @14
                                local.get 0
                                local.get 2
                                i32.sub
                                local.get 2
                                local.get 3
                                i32.add
                                i32.const 0
                                local.get 1
                                i32.sub
                                i32.and
                                i32.add
                                local.set 5
                              end
                              local.get 5
                              local.get 6
                              i32.le_u
                              br_if 5 (;@8;)
                              local.get 5
                              i32.const 2147483646
                              i32.gt_u
                              br_if 5 (;@8;)
                              i32.const 1058280
                              i32.load
                              local.tee 3
                              if  ;; label = @14
                                i32.const 1058272
                                i32.load
                                local.tee 7
                                local.get 5
                                i32.add
                                local.set 1
                                local.get 1
                                local.get 7
                                i32.le_u
                                br_if 6 (;@8;)
                                local.get 1
                                local.get 3
                                i32.gt_u
                                br_if 6 (;@8;)
                              end
                              local.get 2
                              local.get 5
                              call 104
                              local.tee 1
                              i32.ne
                              br_if 1 (;@12;)
                              br 7 (;@6;)
                            end
                            local.get 3
                            local.get 2
                            i32.sub
                            local.get 5
                            i32.and
                            local.tee 5
                            i32.const 2147483646
                            i32.gt_u
                            br_if 4 (;@8;)
                            local.get 5
                            call 104
                            local.set 2
                            local.get 2
                            local.get 1
                            i32.load
                            local.get 1
                            i32.load offset=4
                            i32.add
                            i32.eq
                            br_if 3 (;@9;)
                            local.get 2
                            local.set 1
                          end
                          block  ;; label = @12
                            local.get 5
                            local.get 6
                            i32.const 72
                            i32.add
                            i32.ge_u
                            br_if 0 (;@12;)
                            local.get 1
                            i32.const -1
                            i32.eq
                            br_if 0 (;@12;)
                            i32.const 1058320
                            i32.load
                            local.tee 2
                            local.get 4
                            local.get 5
                            i32.sub
                            i32.add
                            i32.const 0
                            local.get 2
                            i32.sub
                            i32.and
                            local.tee 2
                            i32.const 2147483646
                            i32.gt_u
                            if  ;; label = @13
                              local.get 1
                              local.set 2
                              br 7 (;@6;)
                            end
                            local.get 2
                            call 104
                            i32.const -1
                            i32.ne
                            if  ;; label = @13
                              local.get 2
                              local.get 5
                              i32.add
                              local.set 5
                              local.get 1
                              local.set 2
                              br 7 (;@6;)
                            end
                            i32.const 0
                            local.get 5
                            i32.sub
                            call 104
                            drop
                            br 4 (;@8;)
                          end
                          local.get 1
                          local.tee 2
                          i32.const -1
                          i32.ne
                          br_if 5 (;@6;)
                          br 3 (;@8;)
                        end
                        i32.const 0
                        local.set 1
                        br 8 (;@2;)
                      end
                      i32.const 0
                      local.set 1
                      br 6 (;@3;)
                    end
                    local.get 2
                    i32.const -1
                    i32.ne
                    br_if 2 (;@6;)
                  end
                  i32.const 1058284
                  i32.const 1058284
                  i32.load
                  i32.const 4
                  i32.or
                  i32.store
                end
                local.get 0
                i32.const 2147483646
                i32.gt_u
                br_if 1 (;@5;)
                local.get 0
                call 104
                local.set 2
                i32.const 0
                call 104
                local.set 0
                local.get 2
                i32.const -1
                i32.eq
                br_if 1 (;@5;)
                local.get 0
                i32.const -1
                i32.eq
                br_if 1 (;@5;)
                local.get 0
                local.get 2
                i32.le_u
                br_if 1 (;@5;)
                local.get 0
                local.get 2
                i32.sub
                local.tee 5
                local.get 6
                i32.const 56
                i32.add
                i32.le_u
                br_if 1 (;@5;)
              end
              i32.const 1058272
              i32.const 1058272
              i32.load
              local.get 5
              i32.add
              local.tee 0
              i32.store
              i32.const 1058276
              i32.load
              local.get 0
              i32.lt_u
              if  ;; label = @6
                i32.const 1058276
                local.get 0
                i32.store
              end
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    i32.const 1057864
                    i32.load
                    local.tee 3
                    if  ;; label = @9
                      i32.const 1058288
                      local.set 1
                      loop  ;; label = @10
                        local.get 2
                        local.get 1
                        i32.load
                        local.tee 0
                        local.get 1
                        i32.load offset=4
                        local.tee 4
                        i32.add
                        i32.eq
                        br_if 2 (;@8;)
                        local.get 1
                        i32.load offset=8
                        local.tee 1
                        br_if 0 (;@10;)
                      end
                      br 2 (;@7;)
                    end
                    i32.const 1057856
                    i32.load
                    local.tee 0
                    i32.const 0
                    i32.ne
                    local.get 0
                    local.get 2
                    i32.le_u
                    i32.and
                    i32.eqz
                    if  ;; label = @9
                      i32.const 1057856
                      local.get 2
                      i32.store
                    end
                    i32.const 0
                    local.set 1
                    i32.const 1058292
                    local.get 5
                    i32.store
                    i32.const 1058288
                    local.get 2
                    i32.store
                    i32.const 1057872
                    i32.const -1
                    i32.store
                    i32.const 1057876
                    i32.const 1058312
                    i32.load
                    i32.store
                    i32.const 1058300
                    i32.const 0
                    i32.store
                    loop  ;; label = @9
                      local.get 1
                      i32.const 1057900
                      i32.add
                      local.get 1
                      i32.const 1057888
                      i32.add
                      local.tee 0
                      i32.store
                      local.get 0
                      local.get 1
                      i32.const 1057880
                      i32.add
                      local.tee 4
                      i32.store
                      local.get 1
                      i32.const 1057892
                      i32.add
                      local.get 4
                      i32.store
                      local.get 1
                      i32.const 1057908
                      i32.add
                      local.get 1
                      i32.const 1057896
                      i32.add
                      local.tee 4
                      i32.store
                      local.get 4
                      local.get 0
                      i32.store
                      local.get 1
                      i32.const 1057916
                      i32.add
                      local.get 1
                      i32.const 1057904
                      i32.add
                      local.tee 0
                      i32.store
                      local.get 0
                      local.get 4
                      i32.store
                      local.get 1
                      i32.const 1057912
                      i32.add
                      local.get 0
                      i32.store
                      local.get 1
                      i32.const 32
                      i32.add
                      local.tee 1
                      i32.const 256
                      i32.ne
                      br_if 0 (;@9;)
                    end
                    i32.const -8
                    local.get 2
                    i32.sub
                    i32.const 15
                    i32.and
                    local.tee 0
                    local.get 2
                    i32.add
                    local.tee 1
                    local.get 5
                    i32.const 56
                    i32.sub
                    local.tee 4
                    local.get 0
                    i32.sub
                    local.tee 0
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    i32.const 1057868
                    i32.const 1058328
                    i32.load
                    i32.store
                    i32.const 1057852
                    local.get 0
                    i32.store
                    i32.const 1057864
                    local.get 1
                    i32.store
                    local.get 2
                    local.get 4
                    i32.add
                    i32.const 56
                    i32.store offset=4
                    br 2 (;@6;)
                  end
                  local.get 2
                  local.get 3
                  i32.le_u
                  br_if 0 (;@7;)
                  local.get 0
                  local.get 3
                  i32.gt_u
                  br_if 0 (;@7;)
                  local.get 1
                  i32.load offset=12
                  i32.const 8
                  i32.and
                  br_if 0 (;@7;)
                  i32.const -8
                  local.get 3
                  i32.sub
                  i32.const 15
                  i32.and
                  local.tee 0
                  local.get 3
                  i32.add
                  local.tee 2
                  i32.const 1057852
                  i32.load
                  local.get 5
                  i32.add
                  local.tee 7
                  local.get 0
                  i32.sub
                  local.tee 0
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 1
                  local.get 4
                  local.get 5
                  i32.add
                  i32.store offset=4
                  i32.const 1057868
                  i32.const 1058328
                  i32.load
                  i32.store
                  i32.const 1057852
                  local.get 0
                  i32.store
                  i32.const 1057864
                  local.get 2
                  i32.store
                  local.get 3
                  local.get 7
                  i32.add
                  i32.const 56
                  i32.store offset=4
                  br 1 (;@6;)
                end
                i32.const 1057856
                i32.load
                local.get 2
                i32.gt_u
                if  ;; label = @7
                  i32.const 1057856
                  local.get 2
                  i32.store
                end
                local.get 2
                local.get 5
                i32.add
                local.set 4
                i32.const 1058288
                local.set 1
                block  ;; label = @7
                  loop  ;; label = @8
                    local.get 1
                    i32.load
                    local.tee 0
                    local.get 4
                    i32.ne
                    if  ;; label = @9
                      local.get 1
                      i32.load offset=8
                      local.tee 1
                      br_if 1 (;@8;)
                      br 2 (;@7;)
                    end
                  end
                  local.get 1
                  i32.load8_u offset=12
                  i32.const 8
                  i32.and
                  i32.eqz
                  br_if 3 (;@4;)
                end
                i32.const 1058288
                local.set 1
                loop  ;; label = @7
                  block  ;; label = @8
                    local.get 1
                    i32.load
                    local.tee 0
                    local.get 3
                    i32.le_u
                    if  ;; label = @9
                      local.get 0
                      local.get 1
                      i32.load offset=4
                      i32.add
                      local.tee 4
                      local.get 3
                      i32.gt_u
                      br_if 1 (;@8;)
                    end
                    local.get 1
                    i32.load offset=8
                    local.set 1
                    br 1 (;@7;)
                  end
                end
                i32.const -8
                local.get 2
                i32.sub
                i32.const 15
                i32.and
                local.tee 0
                local.get 2
                i32.add
                local.tee 1
                local.get 5
                i32.const 56
                i32.sub
                local.tee 7
                local.get 0
                i32.sub
                local.tee 8
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 2
                local.get 7
                i32.add
                i32.const 56
                i32.store offset=4
                local.get 3
                local.get 4
                i32.const 55
                local.get 4
                i32.sub
                i32.const 15
                i32.and
                i32.add
                i32.const 63
                i32.sub
                local.tee 0
                local.get 0
                local.get 3
                i32.const 16
                i32.add
                i32.lt_u
                select
                local.tee 0
                i32.const 35
                i32.store offset=4
                i32.const 1057868
                i32.const 1058328
                i32.load
                i32.store
                i32.const 1057852
                local.get 8
                i32.store
                i32.const 1057864
                local.get 1
                i32.store
                local.get 0
                i32.const 16
                i32.add
                i32.const 1058296
                i64.load align=4
                i64.store align=4
                local.get 0
                i32.const 1058288
                i64.load align=4
                i64.store offset=8 align=4
                i32.const 1058296
                local.get 0
                i32.const 8
                i32.add
                i32.store
                i32.const 1058292
                local.get 5
                i32.store
                i32.const 1058288
                local.get 2
                i32.store
                i32.const 1058300
                i32.const 0
                i32.store
                local.get 0
                i32.const 36
                i32.add
                local.set 1
                loop  ;; label = @7
                  local.get 1
                  i32.const 7
                  i32.store
                  local.get 4
                  local.get 1
                  i32.const 4
                  i32.add
                  local.tee 1
                  i32.gt_u
                  br_if 0 (;@7;)
                end
                local.get 0
                local.get 3
                i32.eq
                br_if 0 (;@6;)
                local.get 0
                local.get 0
                i32.load offset=4
                i32.const -2
                i32.and
                i32.store offset=4
                local.get 0
                local.get 0
                local.get 3
                i32.sub
                local.tee 2
                i32.store
                local.get 3
                local.get 2
                i32.const 1
                i32.or
                i32.store offset=4
                block (result i32)  ;; label = @7
                  local.get 2
                  i32.const 255
                  i32.le_u
                  if  ;; label = @8
                    local.get 2
                    i32.const -8
                    i32.and
                    i32.const 1057880
                    i32.add
                    local.set 1
                    block (result i32)  ;; label = @9
                      i32.const 1057840
                      i32.load
                      local.tee 0
                      i32.const 1
                      local.get 2
                      i32.const 3
                      i32.shr_u
                      i32.shl
                      local.tee 2
                      i32.and
                      i32.eqz
                      if  ;; label = @10
                        i32.const 1057840
                        local.get 0
                        local.get 2
                        i32.or
                        i32.store
                        local.get 1
                        br 1 (;@9;)
                      end
                      local.get 1
                      i32.load offset=8
                    end
                    local.tee 0
                    local.get 3
                    i32.store offset=12
                    local.get 1
                    local.get 3
                    i32.store offset=8
                    i32.const 8
                    local.set 4
                    i32.const 12
                    br 1 (;@7;)
                  end
                  i32.const 31
                  local.set 1
                  local.get 2
                  i32.const 16777215
                  i32.le_u
                  if  ;; label = @8
                    local.get 2
                    i32.const 38
                    local.get 2
                    i32.const 8
                    i32.shr_u
                    i32.clz
                    local.tee 0
                    i32.sub
                    i32.shr_u
                    i32.const 1
                    i32.and
                    local.get 0
                    i32.const 1
                    i32.shl
                    i32.sub
                    i32.const 62
                    i32.add
                    local.set 1
                  end
                  local.get 3
                  local.get 1
                  i32.store offset=28
                  local.get 3
                  i64.const 0
                  i64.store offset=16 align=4
                  local.get 1
                  i32.const 2
                  i32.shl
                  i32.const 1058144
                  i32.add
                  local.set 0
                  block  ;; label = @8
                    block  ;; label = @9
                      i32.const 1057844
                      i32.load
                      local.tee 4
                      i32.const 1
                      local.get 1
                      i32.shl
                      local.tee 5
                      i32.and
                      i32.eqz
                      if  ;; label = @10
                        local.get 0
                        local.get 3
                        i32.store
                        i32.const 1057844
                        local.get 4
                        local.get 5
                        i32.or
                        i32.store
                        br 1 (;@9;)
                      end
                      local.get 2
                      i32.const 25
                      local.get 1
                      i32.const 1
                      i32.shr_u
                      i32.sub
                      i32.const 0
                      local.get 1
                      i32.const 31
                      i32.ne
                      select
                      i32.shl
                      local.set 1
                      local.get 0
                      i32.load
                      local.set 4
                      loop  ;; label = @10
                        local.get 4
                        local.tee 0
                        i32.load offset=4
                        i32.const -8
                        i32.and
                        local.get 2
                        i32.eq
                        br_if 2 (;@8;)
                        local.get 1
                        i32.const 29
                        i32.shr_u
                        local.set 4
                        local.get 1
                        i32.const 1
                        i32.shl
                        local.set 1
                        local.get 0
                        local.get 4
                        i32.const 4
                        i32.and
                        i32.add
                        i32.const 16
                        i32.add
                        local.tee 5
                        i32.load
                        local.tee 4
                        br_if 0 (;@10;)
                      end
                      local.get 5
                      local.get 3
                      i32.store
                    end
                    local.get 3
                    local.get 0
                    i32.store offset=24
                    local.get 3
                    local.tee 0
                    local.set 1
                    i32.const 12
                    local.set 4
                    i32.const 8
                    br 1 (;@7;)
                  end
                  local.get 0
                  i32.load offset=8
                  local.set 1
                  local.get 0
                  local.get 3
                  i32.store offset=8
                  local.get 1
                  local.get 3
                  i32.store offset=12
                  local.get 3
                  local.get 1
                  i32.store offset=8
                  i32.const 0
                  local.set 1
                  i32.const 12
                  local.set 4
                  i32.const 24
                end
                local.get 3
                local.get 4
                i32.add
                local.get 0
                i32.store
                local.get 3
                i32.add
                local.get 1
                i32.store
              end
              i32.const 1057852
              i32.load
              local.tee 1
              local.get 6
              i32.le_u
              br_if 0 (;@5;)
              i32.const 1057864
              i32.load
              local.tee 0
              local.get 6
              i32.add
              local.tee 2
              local.get 1
              local.get 6
              i32.sub
              local.tee 1
              i32.const 1
              i32.or
              i32.store offset=4
              i32.const 1057852
              local.get 1
              i32.store
              i32.const 1057864
              local.get 2
              i32.store
              local.get 0
              local.get 6
              i32.const 3
              i32.or
              i32.store offset=4
              local.get 0
              i32.const 8
              i32.add
              local.set 1
              br 4 (;@1;)
            end
            i32.const 0
            local.set 1
            i32.const 1058336
            i32.const 48
            i32.store
            br 3 (;@1;)
          end
          local.get 1
          local.get 2
          i32.store
          local.get 1
          local.get 1
          i32.load offset=4
          local.get 5
          i32.add
          i32.store offset=4
          local.get 2
          i32.const -8
          local.get 2
          i32.sub
          i32.const 15
          i32.and
          i32.add
          local.tee 8
          local.get 6
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 0
          i32.const -8
          local.get 0
          i32.sub
          i32.const 15
          i32.and
          i32.add
          local.tee 5
          local.get 6
          local.get 8
          i32.add
          local.tee 3
          i32.sub
          local.set 6
          block  ;; label = @4
            i32.const 1057864
            i32.load
            local.get 5
            i32.eq
            if  ;; label = @5
              i32.const 1057864
              local.get 3
              i32.store
              i32.const 1057852
              i32.const 1057852
              i32.load
              local.get 6
              i32.add
              local.tee 0
              i32.store
              local.get 3
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              br 1 (;@4;)
            end
            i32.const 1057860
            i32.load
            local.get 5
            i32.eq
            if  ;; label = @5
              i32.const 1057860
              local.get 3
              i32.store
              i32.const 1057848
              i32.const 1057848
              i32.load
              local.get 6
              i32.add
              local.tee 0
              i32.store
              local.get 3
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 3
              i32.add
              local.get 0
              i32.store
              br 1 (;@4;)
            end
            local.get 5
            i32.load offset=4
            local.tee 2
            i32.const 3
            i32.and
            i32.const 1
            i32.eq
            if  ;; label = @5
              local.get 2
              i32.const -8
              i32.and
              local.set 9
              local.get 5
              i32.load offset=12
              local.set 1
              block  ;; label = @6
                local.get 2
                i32.const 255
                i32.le_u
                if  ;; label = @7
                  local.get 5
                  i32.load offset=8
                  local.tee 0
                  local.get 1
                  i32.eq
                  if  ;; label = @8
                    i32.const 1057840
                    i32.const 1057840
                    i32.load
                    i32.const -2
                    local.get 2
                    i32.const 3
                    i32.shr_u
                    i32.rotl
                    i32.and
                    i32.store
                    br 2 (;@6;)
                  end
                  local.get 1
                  local.get 0
                  i32.store offset=8
                  local.get 0
                  local.get 1
                  i32.store offset=12
                  br 1 (;@6;)
                end
                local.get 5
                i32.load offset=24
                local.set 7
                block  ;; label = @7
                  local.get 1
                  local.get 5
                  i32.ne
                  if  ;; label = @8
                    local.get 5
                    i32.load offset=8
                    local.tee 0
                    local.get 1
                    i32.store offset=12
                    local.get 1
                    local.get 0
                    i32.store offset=8
                    br 1 (;@7;)
                  end
                  block  ;; label = @8
                    local.get 5
                    i32.load offset=20
                    local.tee 2
                    if (result i32)  ;; label = @9
                      local.get 5
                      i32.const 20
                      i32.add
                    else
                      local.get 5
                      i32.load offset=16
                      local.tee 2
                      i32.eqz
                      br_if 1 (;@8;)
                      local.get 5
                      i32.const 16
                      i32.add
                    end
                    local.set 0
                    loop  ;; label = @9
                      local.get 0
                      local.set 4
                      local.get 2
                      local.tee 1
                      i32.const 20
                      i32.add
                      local.set 0
                      local.get 1
                      i32.load offset=20
                      local.tee 2
                      br_if 0 (;@9;)
                      local.get 1
                      i32.const 16
                      i32.add
                      local.set 0
                      local.get 1
                      i32.load offset=16
                      local.tee 2
                      br_if 0 (;@9;)
                    end
                    local.get 4
                    i32.const 0
                    i32.store
                    br 1 (;@7;)
                  end
                  i32.const 0
                  local.set 1
                end
                local.get 7
                i32.eqz
                br_if 0 (;@6;)
                block  ;; label = @7
                  local.get 5
                  i32.load offset=28
                  local.tee 0
                  i32.const 2
                  i32.shl
                  i32.const 1058144
                  i32.add
                  local.tee 2
                  i32.load
                  local.get 5
                  i32.eq
                  if  ;; label = @8
                    local.get 2
                    local.get 1
                    i32.store
                    local.get 1
                    br_if 1 (;@7;)
                    i32.const 1057844
                    i32.const 1057844
                    i32.load
                    i32.const -2
                    local.get 0
                    i32.rotl
                    i32.and
                    i32.store
                    br 2 (;@6;)
                  end
                  local.get 7
                  i32.const 16
                  i32.const 20
                  local.get 7
                  i32.load offset=16
                  local.get 5
                  i32.eq
                  select
                  i32.add
                  local.get 1
                  i32.store
                  local.get 1
                  i32.eqz
                  br_if 1 (;@6;)
                end
                local.get 1
                local.get 7
                i32.store offset=24
                local.get 5
                i32.load offset=16
                local.tee 0
                if  ;; label = @7
                  local.get 1
                  local.get 0
                  i32.store offset=16
                  local.get 0
                  local.get 1
                  i32.store offset=24
                end
                local.get 5
                i32.load offset=20
                local.tee 0
                i32.eqz
                br_if 0 (;@6;)
                local.get 1
                local.get 0
                i32.store offset=20
                local.get 0
                local.get 1
                i32.store offset=24
              end
              local.get 6
              local.get 9
              i32.add
              local.set 6
              local.get 5
              local.get 9
              i32.add
              local.tee 5
              i32.load offset=4
              local.set 2
            end
            local.get 5
            local.get 2
            i32.const -2
            i32.and
            i32.store offset=4
            local.get 3
            local.get 6
            i32.add
            local.get 6
            i32.store
            local.get 3
            local.get 6
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 6
            i32.const 255
            i32.le_u
            if  ;; label = @5
              local.get 6
              i32.const -8
              i32.and
              i32.const 1057880
              i32.add
              local.set 0
              block (result i32)  ;; label = @6
                i32.const 1057840
                i32.load
                local.tee 1
                i32.const 1
                local.get 6
                i32.const 3
                i32.shr_u
                i32.shl
                local.tee 2
                i32.and
                i32.eqz
                if  ;; label = @7
                  i32.const 1057840
                  local.get 1
                  local.get 2
                  i32.or
                  i32.store
                  local.get 0
                  br 1 (;@6;)
                end
                local.get 0
                i32.load offset=8
              end
              local.tee 1
              local.get 3
              i32.store offset=12
              local.get 0
              local.get 3
              i32.store offset=8
              local.get 3
              local.get 0
              i32.store offset=12
              local.get 3
              local.get 1
              i32.store offset=8
              br 1 (;@4;)
            end
            i32.const 31
            local.set 1
            local.get 6
            i32.const 16777215
            i32.le_u
            if  ;; label = @5
              local.get 6
              i32.const 38
              local.get 6
              i32.const 8
              i32.shr_u
              i32.clz
              local.tee 0
              i32.sub
              i32.shr_u
              i32.const 1
              i32.and
              local.get 0
              i32.const 1
              i32.shl
              i32.sub
              i32.const 62
              i32.add
              local.set 1
            end
            local.get 3
            local.get 1
            i32.store offset=28
            local.get 3
            i64.const 0
            i64.store offset=16 align=4
            local.get 1
            i32.const 2
            i32.shl
            i32.const 1058144
            i32.add
            local.set 0
            i32.const 1057844
            i32.load
            local.tee 2
            i32.const 1
            local.get 1
            i32.shl
            local.tee 4
            i32.and
            i32.eqz
            if  ;; label = @5
              local.get 0
              local.get 3
              i32.store
              i32.const 1057844
              local.get 2
              local.get 4
              i32.or
              i32.store
              local.get 3
              local.get 0
              i32.store offset=24
              local.get 3
              local.get 3
              i32.store offset=8
              local.get 3
              local.get 3
              i32.store offset=12
              br 1 (;@4;)
            end
            local.get 6
            i32.const 25
            local.get 1
            i32.const 1
            i32.shr_u
            i32.sub
            i32.const 0
            local.get 1
            i32.const 31
            i32.ne
            select
            i32.shl
            local.set 1
            local.get 0
            i32.load
            local.set 0
            block  ;; label = @5
              loop  ;; label = @6
                local.get 0
                local.tee 2
                i32.load offset=4
                i32.const -8
                i32.and
                local.get 6
                i32.eq
                br_if 1 (;@5;)
                local.get 1
                i32.const 29
                i32.shr_u
                local.set 0
                local.get 1
                i32.const 1
                i32.shl
                local.set 1
                local.get 2
                local.get 0
                i32.const 4
                i32.and
                i32.add
                i32.const 16
                i32.add
                local.tee 4
                i32.load
                local.tee 0
                br_if 0 (;@6;)
              end
              local.get 4
              local.get 3
              i32.store
              local.get 3
              local.get 2
              i32.store offset=24
              local.get 3
              local.get 3
              i32.store offset=12
              local.get 3
              local.get 3
              i32.store offset=8
              br 1 (;@4;)
            end
            local.get 2
            i32.load offset=8
            local.tee 0
            local.get 3
            i32.store offset=12
            local.get 2
            local.get 3
            i32.store offset=8
            local.get 3
            i32.const 0
            i32.store offset=24
            local.get 3
            local.get 2
            i32.store offset=12
            local.get 3
            local.get 0
            i32.store offset=8
          end
          local.get 8
          i32.const 8
          i32.add
          local.set 1
          br 2 (;@1;)
        end
        block  ;; label = @3
          local.get 7
          i32.eqz
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 4
            i32.load offset=28
            local.tee 0
            i32.const 2
            i32.shl
            i32.const 1058144
            i32.add
            local.tee 2
            i32.load
            local.get 4
            i32.eq
            if  ;; label = @5
              local.get 2
              local.get 1
              i32.store
              local.get 1
              br_if 1 (;@4;)
              i32.const 1057844
              local.get 8
              i32.const -2
              local.get 0
              i32.rotl
              i32.and
              local.tee 8
              i32.store
              br 2 (;@3;)
            end
            local.get 7
            i32.const 16
            i32.const 20
            local.get 7
            i32.load offset=16
            local.get 4
            i32.eq
            select
            i32.add
            local.get 1
            i32.store
            local.get 1
            i32.eqz
            br_if 1 (;@3;)
          end
          local.get 1
          local.get 7
          i32.store offset=24
          local.get 4
          i32.load offset=16
          local.tee 0
          if  ;; label = @4
            local.get 1
            local.get 0
            i32.store offset=16
            local.get 0
            local.get 1
            i32.store offset=24
          end
          local.get 4
          i32.load offset=20
          local.tee 0
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          local.get 0
          i32.store offset=20
          local.get 0
          local.get 1
          i32.store offset=24
        end
        block  ;; label = @3
          local.get 6
          i32.const 15
          i32.le_u
          if  ;; label = @4
            local.get 4
            local.get 3
            local.get 6
            i32.or
            local.tee 0
            i32.const 3
            i32.or
            i32.store offset=4
            local.get 0
            local.get 4
            i32.add
            local.tee 0
            local.get 0
            i32.load offset=4
            i32.const 1
            i32.or
            i32.store offset=4
            br 1 (;@3;)
          end
          local.get 3
          local.get 4
          i32.add
          local.tee 5
          local.get 6
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 4
          local.get 3
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 5
          local.get 6
          i32.add
          local.get 6
          i32.store
          local.get 6
          i32.const 255
          i32.le_u
          if  ;; label = @4
            local.get 6
            i32.const -8
            i32.and
            i32.const 1057880
            i32.add
            local.set 0
            block (result i32)  ;; label = @5
              i32.const 1057840
              i32.load
              local.tee 1
              i32.const 1
              local.get 6
              i32.const 3
              i32.shr_u
              i32.shl
              local.tee 2
              i32.and
              i32.eqz
              if  ;; label = @6
                i32.const 1057840
                local.get 1
                local.get 2
                i32.or
                i32.store
                local.get 0
                br 1 (;@5;)
              end
              local.get 0
              i32.load offset=8
            end
            local.tee 1
            local.get 5
            i32.store offset=12
            local.get 0
            local.get 5
            i32.store offset=8
            local.get 5
            local.get 0
            i32.store offset=12
            local.get 5
            local.get 1
            i32.store offset=8
            br 1 (;@3;)
          end
          i32.const 31
          local.set 1
          local.get 6
          i32.const 16777215
          i32.le_u
          if  ;; label = @4
            local.get 6
            i32.const 38
            local.get 6
            i32.const 8
            i32.shr_u
            i32.clz
            local.tee 0
            i32.sub
            i32.shr_u
            i32.const 1
            i32.and
            local.get 0
            i32.const 1
            i32.shl
            i32.sub
            i32.const 62
            i32.add
            local.set 1
          end
          local.get 5
          local.get 1
          i32.store offset=28
          local.get 5
          i64.const 0
          i64.store offset=16 align=4
          local.get 1
          i32.const 2
          i32.shl
          i32.const 1058144
          i32.add
          local.set 0
          i32.const 1
          local.get 1
          i32.shl
          local.tee 2
          local.get 8
          i32.and
          i32.eqz
          if  ;; label = @4
            local.get 0
            local.get 5
            i32.store
            i32.const 1057844
            local.get 2
            local.get 8
            i32.or
            i32.store
            local.get 5
            local.get 0
            i32.store offset=24
            local.get 5
            local.get 5
            i32.store offset=8
            local.get 5
            local.get 5
            i32.store offset=12
            br 1 (;@3;)
          end
          local.get 6
          i32.const 25
          local.get 1
          i32.const 1
          i32.shr_u
          i32.sub
          i32.const 0
          local.get 1
          i32.const 31
          i32.ne
          select
          i32.shl
          local.set 1
          local.get 0
          i32.load
          local.set 0
          block  ;; label = @4
            loop  ;; label = @5
              local.get 0
              local.tee 2
              i32.load offset=4
              i32.const -8
              i32.and
              local.get 6
              i32.eq
              br_if 1 (;@4;)
              local.get 1
              i32.const 29
              i32.shr_u
              local.set 0
              local.get 1
              i32.const 1
              i32.shl
              local.set 1
              local.get 2
              local.get 0
              i32.const 4
              i32.and
              i32.add
              i32.const 16
              i32.add
              local.tee 3
              i32.load
              local.tee 0
              br_if 0 (;@5;)
            end
            local.get 3
            local.get 5
            i32.store
            local.get 5
            local.get 2
            i32.store offset=24
            local.get 5
            local.get 5
            i32.store offset=12
            local.get 5
            local.get 5
            i32.store offset=8
            br 1 (;@3;)
          end
          local.get 2
          i32.load offset=8
          local.tee 0
          local.get 5
          i32.store offset=12
          local.get 2
          local.get 5
          i32.store offset=8
          local.get 5
          i32.const 0
          i32.store offset=24
          local.get 5
          local.get 2
          i32.store offset=12
          local.get 5
          local.get 0
          i32.store offset=8
        end
        local.get 4
        i32.const 8
        i32.add
        local.set 1
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 9
        i32.eqz
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 2
          i32.load offset=28
          local.tee 0
          i32.const 2
          i32.shl
          i32.const 1058144
          i32.add
          local.tee 4
          i32.load
          local.get 2
          i32.eq
          if  ;; label = @4
            local.get 4
            local.get 1
            i32.store
            local.get 1
            br_if 1 (;@3;)
            i32.const 1057844
            local.get 11
            i32.const -2
            local.get 0
            i32.rotl
            i32.and
            i32.store
            br 2 (;@2;)
          end
          local.get 9
          i32.const 16
          i32.const 20
          local.get 9
          i32.load offset=16
          local.get 2
          i32.eq
          select
          i32.add
          local.get 1
          i32.store
          local.get 1
          i32.eqz
          br_if 1 (;@2;)
        end
        local.get 1
        local.get 9
        i32.store offset=24
        local.get 2
        i32.load offset=16
        local.tee 0
        if  ;; label = @3
          local.get 1
          local.get 0
          i32.store offset=16
          local.get 0
          local.get 1
          i32.store offset=24
        end
        local.get 2
        i32.load offset=20
        local.tee 0
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 0
        i32.store offset=20
        local.get 0
        local.get 1
        i32.store offset=24
      end
      block  ;; label = @2
        local.get 6
        i32.const 15
        i32.le_u
        if  ;; label = @3
          local.get 2
          local.get 3
          local.get 6
          i32.or
          local.tee 0
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 0
          local.get 2
          i32.add
          local.tee 0
          local.get 0
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          br 1 (;@2;)
        end
        local.get 2
        local.get 3
        i32.add
        local.tee 7
        local.get 6
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 2
        local.get 3
        i32.const 3
        i32.or
        i32.store offset=4
        local.get 6
        local.get 7
        i32.add
        local.get 6
        i32.store
        local.get 8
        if  ;; label = @3
          local.get 8
          i32.const -8
          i32.and
          i32.const 1057880
          i32.add
          local.set 0
          i32.const 1057860
          i32.load
          local.set 4
          block (result i32)  ;; label = @4
            i32.const 1
            local.get 8
            i32.const 3
            i32.shr_u
            i32.shl
            local.tee 1
            local.get 5
            i32.and
            i32.eqz
            if  ;; label = @5
              i32.const 1057840
              local.get 1
              local.get 5
              i32.or
              i32.store
              local.get 0
              br 1 (;@4;)
            end
            local.get 0
            i32.load offset=8
          end
          local.tee 1
          local.get 4
          i32.store offset=12
          local.get 0
          local.get 4
          i32.store offset=8
          local.get 4
          local.get 0
          i32.store offset=12
          local.get 4
          local.get 1
          i32.store offset=8
        end
        i32.const 1057860
        local.get 7
        i32.store
        i32.const 1057848
        local.get 6
        i32.store
      end
      local.get 2
      i32.const 8
      i32.add
      local.set 1
    end
    local.get 10
    i32.const 16
    i32.add
    global.set 0
    local.get 1)
  (func (;98;) (type 2) (param i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      local.get 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.const 8
      i32.sub
      local.set 3
      local.get 3
      local.get 0
      i32.const 4
      i32.sub
      i32.load
      local.tee 1
      i32.const -8
      i32.and
      local.tee 0
      i32.add
      local.set 5
      block  ;; label = @2
        local.get 1
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        local.get 1
        i32.const 2
        i32.and
        i32.eqz
        br_if 1 (;@1;)
        local.get 3
        local.get 3
        i32.load
        local.tee 1
        i32.sub
        local.tee 3
        i32.const 1057856
        i32.load
        i32.lt_u
        br_if 1 (;@1;)
        local.get 0
        local.get 1
        i32.add
        local.set 0
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              i32.const 1057860
              i32.load
              local.get 3
              i32.ne
              if  ;; label = @6
                local.get 3
                i32.load offset=12
                local.set 2
                local.get 1
                i32.const 255
                i32.le_u
                if  ;; label = @7
                  local.get 3
                  i32.load offset=8
                  local.tee 4
                  local.get 2
                  i32.ne
                  br_if 2 (;@5;)
                  i32.const 1057840
                  i32.const 1057840
                  i32.load
                  i32.const -2
                  local.get 1
                  i32.const 3
                  i32.shr_u
                  i32.rotl
                  i32.and
                  i32.store
                  br 5 (;@2;)
                end
                local.get 3
                i32.load offset=24
                local.set 6
                local.get 2
                local.get 3
                i32.ne
                if  ;; label = @7
                  local.get 3
                  i32.load offset=8
                  local.tee 1
                  local.get 2
                  i32.store offset=12
                  local.get 2
                  local.get 1
                  i32.store offset=8
                  br 4 (;@3;)
                end
                local.get 3
                i32.load offset=20
                local.tee 1
                if (result i32)  ;; label = @7
                  local.get 3
                  i32.const 20
                  i32.add
                else
                  local.get 3
                  i32.load offset=16
                  local.tee 1
                  i32.eqz
                  br_if 3 (;@4;)
                  local.get 3
                  i32.const 16
                  i32.add
                end
                local.set 4
                loop  ;; label = @7
                  local.get 4
                  local.set 7
                  local.get 1
                  local.tee 2
                  i32.const 20
                  i32.add
                  local.set 4
                  local.get 2
                  i32.load offset=20
                  local.tee 1
                  br_if 0 (;@7;)
                  local.get 2
                  i32.const 16
                  i32.add
                  local.set 4
                  local.get 2
                  i32.load offset=16
                  local.tee 1
                  br_if 0 (;@7;)
                end
                local.get 7
                i32.const 0
                i32.store
                br 3 (;@3;)
              end
              local.get 5
              i32.load offset=4
              local.tee 1
              i32.const 3
              i32.and
              i32.const 3
              i32.ne
              br_if 3 (;@2;)
              local.get 5
              local.get 1
              i32.const -2
              i32.and
              i32.store offset=4
              i32.const 1057848
              local.get 0
              i32.store
              local.get 5
              local.get 0
              i32.store
              local.get 3
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              return
            end
            local.get 2
            local.get 4
            i32.store offset=8
            local.get 4
            local.get 2
            i32.store offset=12
            br 2 (;@2;)
          end
          i32.const 0
          local.set 2
        end
        local.get 6
        i32.eqz
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 3
          i32.load offset=28
          local.tee 1
          i32.const 2
          i32.shl
          i32.const 1058144
          i32.add
          local.tee 4
          i32.load
          local.get 3
          i32.eq
          if  ;; label = @4
            local.get 4
            local.get 2
            i32.store
            local.get 2
            br_if 1 (;@3;)
            i32.const 1057844
            i32.const 1057844
            i32.load
            i32.const -2
            local.get 1
            i32.rotl
            i32.and
            i32.store
            br 2 (;@2;)
          end
          local.get 6
          i32.const 16
          i32.const 20
          local.get 6
          i32.load offset=16
          local.get 3
          i32.eq
          select
          i32.add
          local.get 2
          i32.store
          local.get 2
          i32.eqz
          br_if 1 (;@2;)
        end
        local.get 2
        local.get 6
        i32.store offset=24
        local.get 3
        i32.load offset=16
        local.tee 1
        if  ;; label = @3
          local.get 2
          local.get 1
          i32.store offset=16
          local.get 1
          local.get 2
          i32.store offset=24
        end
        local.get 3
        i32.load offset=20
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 1
        i32.store offset=20
        local.get 1
        local.get 2
        i32.store offset=24
      end
      local.get 3
      local.get 5
      i32.ge_u
      br_if 0 (;@1;)
      local.get 5
      i32.load offset=4
      local.tee 1
      i32.const 1
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 1
              i32.const 2
              i32.and
              i32.eqz
              if  ;; label = @6
                i32.const 1057864
                i32.load
                local.get 5
                i32.eq
                if  ;; label = @7
                  i32.const 1057864
                  local.get 3
                  i32.store
                  i32.const 1057852
                  i32.const 1057852
                  i32.load
                  local.get 0
                  i32.add
                  local.tee 0
                  i32.store
                  local.get 3
                  local.get 0
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 3
                  i32.const 1057860
                  i32.load
                  i32.ne
                  br_if 6 (;@1;)
                  i32.const 1057848
                  i32.const 0
                  i32.store
                  i32.const 1057860
                  i32.const 0
                  i32.store
                  return
                end
                i32.const 1057860
                i32.load
                local.get 5
                i32.eq
                if  ;; label = @7
                  i32.const 1057860
                  local.get 3
                  i32.store
                  i32.const 1057848
                  i32.const 1057848
                  i32.load
                  local.get 0
                  i32.add
                  local.tee 0
                  i32.store
                  local.get 3
                  local.get 0
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 3
                  i32.add
                  local.get 0
                  i32.store
                  return
                end
                local.get 1
                i32.const -8
                i32.and
                local.get 0
                i32.add
                local.set 0
                local.get 5
                i32.load offset=12
                local.set 2
                local.get 1
                i32.const 255
                i32.le_u
                if  ;; label = @7
                  local.get 2
                  local.get 5
                  i32.load offset=8
                  local.tee 4
                  i32.eq
                  if  ;; label = @8
                    i32.const 1057840
                    i32.const 1057840
                    i32.load
                    i32.const -2
                    local.get 1
                    i32.const 3
                    i32.shr_u
                    i32.rotl
                    i32.and
                    i32.store
                    br 5 (;@3;)
                  end
                  local.get 2
                  local.get 4
                  i32.store offset=8
                  local.get 4
                  local.get 2
                  i32.store offset=12
                  br 4 (;@3;)
                end
                local.get 5
                i32.load offset=24
                local.set 6
                local.get 2
                local.get 5
                i32.ne
                if  ;; label = @7
                  local.get 5
                  i32.load offset=8
                  local.tee 1
                  local.get 2
                  i32.store offset=12
                  local.get 2
                  local.get 1
                  i32.store offset=8
                  br 3 (;@4;)
                end
                local.get 5
                i32.load offset=20
                local.tee 1
                if (result i32)  ;; label = @7
                  local.get 5
                  i32.const 20
                  i32.add
                else
                  local.get 5
                  i32.load offset=16
                  local.tee 1
                  i32.eqz
                  br_if 2 (;@5;)
                  local.get 5
                  i32.const 16
                  i32.add
                end
                local.set 4
                loop  ;; label = @7
                  local.get 4
                  local.set 7
                  local.get 1
                  local.tee 2
                  i32.const 20
                  i32.add
                  local.set 4
                  local.get 2
                  i32.load offset=20
                  local.tee 1
                  br_if 0 (;@7;)
                  local.get 2
                  i32.const 16
                  i32.add
                  local.set 4
                  local.get 2
                  i32.load offset=16
                  local.tee 1
                  br_if 0 (;@7;)
                end
                local.get 7
                i32.const 0
                i32.store
                br 2 (;@4;)
              end
              local.get 5
              local.get 1
              i32.const -2
              i32.and
              i32.store offset=4
              local.get 0
              local.get 3
              i32.add
              local.get 0
              i32.store
              local.get 3
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              br 3 (;@2;)
            end
            i32.const 0
            local.set 2
          end
          local.get 6
          i32.eqz
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 5
            i32.load offset=28
            local.tee 1
            i32.const 2
            i32.shl
            i32.const 1058144
            i32.add
            local.tee 4
            i32.load
            local.get 5
            i32.eq
            if  ;; label = @5
              local.get 4
              local.get 2
              i32.store
              local.get 2
              br_if 1 (;@4;)
              i32.const 1057844
              i32.const 1057844
              i32.load
              i32.const -2
              local.get 1
              i32.rotl
              i32.and
              i32.store
              br 2 (;@3;)
            end
            local.get 6
            i32.const 16
            i32.const 20
            local.get 6
            i32.load offset=16
            local.get 5
            i32.eq
            select
            i32.add
            local.get 2
            i32.store
            local.get 2
            i32.eqz
            br_if 1 (;@3;)
          end
          local.get 2
          local.get 6
          i32.store offset=24
          local.get 5
          i32.load offset=16
          local.tee 1
          if  ;; label = @4
            local.get 2
            local.get 1
            i32.store offset=16
            local.get 1
            local.get 2
            i32.store offset=24
          end
          local.get 5
          i32.load offset=20
          local.tee 1
          i32.eqz
          br_if 0 (;@3;)
          local.get 2
          local.get 1
          i32.store offset=20
          local.get 1
          local.get 2
          i32.store offset=24
        end
        local.get 0
        local.get 3
        i32.add
        local.get 0
        i32.store
        local.get 3
        local.get 0
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 3
        i32.const 1057860
        i32.load
        i32.ne
        br_if 0 (;@2;)
        i32.const 1057848
        local.get 0
        i32.store
        return
      end
      local.get 0
      i32.const 255
      i32.le_u
      if  ;; label = @2
        local.get 0
        i32.const -8
        i32.and
        i32.const 1057880
        i32.add
        local.set 1
        block (result i32)  ;; label = @3
          i32.const 1057840
          i32.load
          local.tee 4
          i32.const 1
          local.get 0
          i32.const 3
          i32.shr_u
          i32.shl
          local.tee 0
          i32.and
          i32.eqz
          if  ;; label = @4
            i32.const 1057840
            local.get 0
            local.get 4
            i32.or
            i32.store
            local.get 1
            br 1 (;@3;)
          end
          local.get 1
          i32.load offset=8
        end
        local.tee 0
        local.get 3
        i32.store offset=12
        local.get 1
        local.get 3
        i32.store offset=8
        local.get 3
        local.get 1
        i32.store offset=12
        local.get 3
        local.get 0
        i32.store offset=8
        return
      end
      i32.const 31
      local.set 2
      local.get 0
      i32.const 16777215
      i32.le_u
      if  ;; label = @2
        local.get 0
        i32.const 38
        local.get 0
        i32.const 8
        i32.shr_u
        i32.clz
        local.tee 1
        i32.sub
        i32.shr_u
        i32.const 1
        i32.and
        local.get 1
        i32.const 1
        i32.shl
        i32.sub
        i32.const 62
        i32.add
        local.set 2
      end
      local.get 3
      local.get 2
      i32.store offset=28
      local.get 3
      i64.const 0
      i64.store offset=16 align=4
      local.get 2
      i32.const 2
      i32.shl
      i32.const 1058144
      i32.add
      local.set 7
      block (result i32)  ;; label = @2
        block  ;; label = @3
          block (result i32)  ;; label = @4
            i32.const 1057844
            i32.load
            local.tee 1
            i32.const 1
            local.get 2
            i32.shl
            local.tee 4
            i32.and
            i32.eqz
            if  ;; label = @5
              i32.const 1057844
              local.get 1
              local.get 4
              i32.or
              i32.store
              i32.const 24
              local.set 2
              local.get 7
              local.set 4
              i32.const 8
              br 1 (;@4;)
            end
            local.get 0
            i32.const 25
            local.get 2
            i32.const 1
            i32.shr_u
            i32.sub
            i32.const 0
            local.get 2
            i32.const 31
            i32.ne
            select
            i32.shl
            local.set 2
            local.get 7
            i32.load
            local.set 4
            loop  ;; label = @5
              local.get 4
              local.tee 1
              i32.load offset=4
              i32.const -8
              i32.and
              local.get 0
              i32.eq
              br_if 2 (;@3;)
              local.get 2
              i32.const 29
              i32.shr_u
              local.set 4
              local.get 2
              i32.const 1
              i32.shl
              local.set 2
              local.get 1
              local.get 4
              i32.const 4
              i32.and
              i32.add
              i32.const 16
              i32.add
              local.tee 7
              i32.load
              local.tee 4
              br_if 0 (;@5;)
            end
            i32.const 24
            local.set 2
            local.get 1
            local.set 4
            i32.const 8
          end
          local.set 0
          local.get 3
          local.tee 1
          br 1 (;@2;)
        end
        local.get 1
        i32.load offset=8
        local.tee 4
        local.get 3
        i32.store offset=12
        i32.const 8
        local.set 2
        local.get 1
        i32.const 8
        i32.add
        local.set 7
        i32.const 24
        local.set 0
        i32.const 0
      end
      local.set 5
      local.get 7
      local.get 3
      i32.store
      local.get 2
      local.get 3
      i32.add
      local.get 4
      i32.store
      local.get 3
      local.get 1
      i32.store offset=12
      local.get 0
      local.get 3
      i32.add
      local.get 5
      i32.store
      i32.const 1057872
      i32.const 1057872
      i32.load
      i32.const 1
      i32.sub
      local.tee 0
      i32.const -1
      local.get 0
      select
      i32.store
    end)
  (func (;99;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i64)
    block  ;; label = @1
      block (result i32)  ;; label = @2
        i32.const 0
        local.get 0
        i32.eqz
        br_if 0 (;@2;)
        drop
        local.get 0
        i64.extend_i32_u
        local.get 1
        i64.extend_i32_u
        i64.mul
        local.tee 4
        i32.wrap_i64
        local.tee 2
        local.get 0
        local.get 1
        i32.or
        i32.const 65536
        i32.lt_u
        br_if 0 (;@2;)
        drop
        i32.const -1
        local.get 2
        local.get 4
        i64.const 32
        i64.shr_u
        i32.wrap_i64
        select
      end
      local.tee 2
      call 97
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.const 4
      i32.sub
      i32.load8_u
      i32.const 3
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 2
        i32.const 33
        i32.ge_u
        if  ;; label = @3
          local.get 0
          i32.const 0
          local.get 2
          memory.fill
          br 1 (;@2;)
        end
        block  ;; label = @3
          local.get 2
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          i32.const 0
          i32.store8
          local.get 0
          local.get 2
          i32.add
          local.tee 1
          i32.const 1
          i32.sub
          i32.const 0
          i32.store8
          local.get 2
          i32.const 3
          i32.lt_u
          br_if 0 (;@3;)
          local.get 0
          i32.const 0
          i32.store8 offset=2
          local.get 0
          i32.const 0
          i32.store8 offset=1
          local.get 1
          i32.const 3
          i32.sub
          i32.const 0
          i32.store8
          local.get 1
          i32.const 2
          i32.sub
          i32.const 0
          i32.store8
          local.get 2
          i32.const 7
          i32.lt_u
          br_if 0 (;@3;)
          local.get 0
          i32.const 0
          i32.store8 offset=3
          local.get 1
          i32.const 4
          i32.sub
          i32.const 0
          i32.store8
          local.get 2
          i32.const 9
          i32.lt_u
          br_if 0 (;@3;)
          i32.const 0
          local.get 0
          i32.sub
          i32.const 3
          i32.and
          local.tee 1
          local.get 0
          i32.add
          local.tee 3
          i32.const 0
          i32.store
          local.get 2
          local.get 1
          i32.sub
          i32.const 60
          i32.and
          local.tee 1
          local.get 3
          i32.add
          local.tee 2
          i32.const 4
          i32.sub
          i32.const 0
          i32.store
          local.get 1
          i32.const 9
          i32.lt_u
          br_if 0 (;@3;)
          local.get 3
          i32.const 0
          i32.store offset=8
          local.get 3
          i32.const 0
          i32.store offset=4
          local.get 2
          i32.const 8
          i32.sub
          i32.const 0
          i32.store
          local.get 2
          i32.const 12
          i32.sub
          i32.const 0
          i32.store
          local.get 1
          i32.const 25
          i32.lt_u
          br_if 0 (;@3;)
          local.get 3
          i32.const 0
          i32.store offset=24
          local.get 3
          i32.const 0
          i32.store offset=20
          local.get 3
          i32.const 0
          i32.store offset=16
          local.get 3
          i32.const 0
          i32.store offset=12
          local.get 2
          i32.const 16
          i32.sub
          i32.const 0
          i32.store
          local.get 2
          i32.const 20
          i32.sub
          i32.const 0
          i32.store
          local.get 2
          i32.const 24
          i32.sub
          i32.const 0
          i32.store
          local.get 2
          i32.const 28
          i32.sub
          i32.const 0
          i32.store
          local.get 1
          local.get 3
          i32.const 4
          i32.and
          i32.const 24
          i32.or
          local.tee 1
          i32.sub
          local.tee 2
          i32.const 32
          i32.lt_u
          br_if 0 (;@3;)
          local.get 1
          local.get 3
          i32.add
          local.set 1
          loop  ;; label = @4
            local.get 1
            i64.const 0
            i64.store offset=24
            local.get 1
            i64.const 0
            i64.store offset=16
            local.get 1
            i64.const 0
            i64.store offset=8
            local.get 1
            i64.const 0
            i64.store
            local.get 1
            i32.const 32
            i32.add
            local.set 1
            local.get 2
            i32.const 32
            i32.sub
            local.tee 2
            i32.const 31
            i32.gt_u
            br_if 0 (;@4;)
          end
        end
      end
    end
    local.get 0)
  (func (;100;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 0
    i32.eqz
    if  ;; label = @1
      local.get 1
      call 97
      return
    end
    local.get 1
    i32.const -64
    i32.ge_u
    if  ;; label = @1
      i32.const 1058336
      i32.const 48
      i32.store
      i32.const 0
      return
    end
    i32.const 16
    local.get 1
    i32.const 19
    i32.add
    i32.const -16
    i32.and
    local.get 1
    i32.const 11
    i32.lt_u
    select
    local.set 3
    local.get 0
    i32.const 4
    i32.sub
    local.tee 7
    i32.load
    local.tee 8
    i32.const -8
    i32.and
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        local.get 8
        i32.const 3
        i32.and
        i32.eqz
        if  ;; label = @3
          local.get 3
          i32.const 256
          i32.lt_u
          br_if 1 (;@2;)
          local.get 2
          local.get 3
          i32.const 4
          i32.or
          i32.lt_u
          br_if 1 (;@2;)
          local.get 2
          local.get 3
          i32.sub
          i32.const 1058320
          i32.load
          i32.const 1
          i32.shl
          i32.le_u
          br_if 2 (;@1;)
          br 1 (;@2;)
        end
        local.get 2
        local.get 0
        i32.const 8
        i32.sub
        local.tee 6
        i32.add
        local.set 4
        local.get 2
        local.get 3
        i32.ge_u
        if  ;; label = @3
          local.get 2
          local.get 3
          i32.sub
          local.tee 1
          i32.const 16
          i32.lt_u
          br_if 2 (;@1;)
          local.get 7
          local.get 3
          local.get 8
          i32.const 1
          i32.and
          i32.or
          i32.const 2
          i32.or
          i32.store
          local.get 3
          local.get 6
          i32.add
          local.tee 2
          local.get 1
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 4
          local.get 4
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 2
          local.get 1
          call 101
          local.get 0
          return
        end
        i32.const 1057864
        i32.load
        local.get 4
        i32.eq
        if  ;; label = @3
          i32.const 1057852
          i32.load
          local.get 2
          i32.add
          local.tee 2
          local.get 3
          i32.le_u
          br_if 1 (;@2;)
          local.get 7
          local.get 3
          local.get 8
          i32.const 1
          i32.and
          i32.or
          i32.const 2
          i32.or
          i32.store
          i32.const 1057864
          local.get 3
          local.get 6
          i32.add
          local.tee 1
          i32.store
          i32.const 1057852
          local.get 2
          local.get 3
          i32.sub
          local.tee 2
          i32.store
          local.get 1
          local.get 2
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 0
          return
        end
        i32.const 1057860
        i32.load
        local.get 4
        i32.eq
        if  ;; label = @3
          i32.const 1057848
          i32.load
          local.get 2
          i32.add
          local.tee 2
          local.get 3
          i32.lt_u
          br_if 1 (;@2;)
          block  ;; label = @4
            local.get 2
            local.get 3
            i32.sub
            local.tee 1
            i32.const 16
            i32.ge_u
            if  ;; label = @5
              local.get 7
              local.get 3
              local.get 8
              i32.const 1
              i32.and
              i32.or
              i32.const 2
              i32.or
              i32.store
              local.get 3
              local.get 6
              i32.add
              local.tee 5
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 2
              local.get 6
              i32.add
              local.tee 2
              local.get 1
              i32.store
              local.get 2
              local.get 2
              i32.load offset=4
              i32.const -2
              i32.and
              i32.store offset=4
              br 1 (;@4;)
            end
            local.get 7
            local.get 8
            i32.const 1
            i32.and
            local.get 2
            i32.or
            i32.const 2
            i32.or
            i32.store
            local.get 2
            local.get 6
            i32.add
            local.tee 1
            local.get 1
            i32.load offset=4
            i32.const 1
            i32.or
            i32.store offset=4
            i32.const 0
            local.set 1
          end
          i32.const 1057860
          local.get 5
          i32.store
          i32.const 1057848
          local.get 1
          i32.store
          local.get 0
          return
        end
        local.get 4
        i32.load offset=4
        local.tee 5
        i32.const 2
        i32.and
        br_if 0 (;@2;)
        local.get 3
        local.get 5
        i32.const -8
        i32.and
        local.get 2
        i32.add
        local.tee 9
        i32.gt_u
        br_if 0 (;@2;)
        local.get 9
        local.get 3
        i32.sub
        local.set 11
        local.get 4
        i32.load offset=12
        local.set 1
        block  ;; label = @3
          local.get 5
          i32.const 255
          i32.le_u
          if  ;; label = @4
            local.get 1
            local.get 4
            i32.load offset=8
            local.tee 2
            i32.eq
            if  ;; label = @5
              i32.const 1057840
              i32.const 1057840
              i32.load
              i32.const -2
              local.get 5
              i32.const 3
              i32.shr_u
              i32.rotl
              i32.and
              i32.store
              br 2 (;@3;)
            end
            local.get 1
            local.get 2
            i32.store offset=8
            local.get 2
            local.get 1
            i32.store offset=12
            br 1 (;@3;)
          end
          local.get 4
          i32.load offset=24
          local.set 10
          block  ;; label = @4
            local.get 1
            local.get 4
            i32.ne
            if  ;; label = @5
              local.get 4
              i32.load offset=8
              local.tee 2
              local.get 1
              i32.store offset=12
              local.get 1
              local.get 2
              i32.store offset=8
              br 1 (;@4;)
            end
            block  ;; label = @5
              local.get 4
              i32.load offset=20
              local.tee 2
              if (result i32)  ;; label = @6
                local.get 4
                i32.const 20
                i32.add
              else
                local.get 4
                i32.load offset=16
                local.tee 2
                i32.eqz
                br_if 1 (;@5;)
                local.get 4
                i32.const 16
                i32.add
              end
              local.set 5
              loop  ;; label = @6
                local.get 5
                local.set 12
                local.get 2
                local.tee 1
                i32.const 20
                i32.add
                local.set 5
                local.get 1
                i32.load offset=20
                local.tee 2
                br_if 0 (;@6;)
                local.get 1
                i32.const 16
                i32.add
                local.set 5
                local.get 1
                i32.load offset=16
                local.tee 2
                br_if 0 (;@6;)
              end
              local.get 12
              i32.const 0
              i32.store
              br 1 (;@4;)
            end
            i32.const 0
            local.set 1
          end
          local.get 10
          i32.eqz
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 4
            i32.load offset=28
            local.tee 2
            i32.const 2
            i32.shl
            i32.const 1058144
            i32.add
            local.tee 5
            i32.load
            local.get 4
            i32.eq
            if  ;; label = @5
              local.get 5
              local.get 1
              i32.store
              local.get 1
              br_if 1 (;@4;)
              i32.const 1057844
              i32.const 1057844
              i32.load
              i32.const -2
              local.get 2
              i32.rotl
              i32.and
              i32.store
              br 2 (;@3;)
            end
            local.get 10
            i32.const 16
            i32.const 20
            local.get 10
            i32.load offset=16
            local.get 4
            i32.eq
            select
            i32.add
            local.get 1
            i32.store
            local.get 1
            i32.eqz
            br_if 1 (;@3;)
          end
          local.get 1
          local.get 10
          i32.store offset=24
          local.get 4
          i32.load offset=16
          local.tee 2
          if  ;; label = @4
            local.get 1
            local.get 2
            i32.store offset=16
            local.get 2
            local.get 1
            i32.store offset=24
          end
          local.get 4
          i32.load offset=20
          local.tee 2
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          local.get 2
          i32.store offset=20
          local.get 2
          local.get 1
          i32.store offset=24
        end
        local.get 11
        i32.const 15
        i32.le_u
        if  ;; label = @3
          local.get 7
          local.get 8
          i32.const 1
          i32.and
          local.get 9
          i32.or
          i32.const 2
          i32.or
          i32.store
          local.get 6
          local.get 9
          i32.add
          local.tee 1
          local.get 1
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 0
          return
        end
        local.get 7
        local.get 3
        local.get 8
        i32.const 1
        i32.and
        i32.or
        i32.const 2
        i32.or
        i32.store
        local.get 3
        local.get 6
        i32.add
        local.tee 1
        local.get 11
        i32.const 3
        i32.or
        i32.store offset=4
        local.get 6
        local.get 9
        i32.add
        local.tee 2
        local.get 2
        i32.load offset=4
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 1
        local.get 11
        call 101
        local.get 0
        return
      end
      local.get 1
      call 97
      local.tee 2
      i32.eqz
      if  ;; label = @2
        i32.const 0
        return
      end
      local.get 2
      local.get 0
      local.get 7
      i32.load
      local.tee 2
      i32.const -8
      i32.and
      i32.const -4
      i32.const -8
      local.get 2
      i32.const 3
      i32.and
      select
      i32.add
      local.tee 2
      local.get 1
      local.get 1
      local.get 2
      i32.gt_u
      select
      call 106
      local.get 0
      call 98
      local.set 0
    end
    local.get 0)
  (func (;101;) (type 3) (param i32 i32)
    (local i32 i32 i32 i32 i32 i32)
    local.get 0
    local.get 1
    i32.add
    local.set 5
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.load offset=4
        local.tee 2
        i32.const 1
        i32.and
        br_if 0 (;@2;)
        local.get 2
        i32.const 2
        i32.and
        i32.eqz
        br_if 1 (;@1;)
        local.get 0
        i32.load
        local.tee 2
        local.get 1
        i32.add
        local.set 1
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              local.get 2
              i32.sub
              local.tee 0
              i32.const 1057860
              i32.load
              i32.ne
              if  ;; label = @6
                local.get 0
                i32.load offset=12
                local.set 3
                local.get 2
                i32.const 255
                i32.le_u
                if  ;; label = @7
                  local.get 0
                  i32.load offset=8
                  local.tee 4
                  local.get 3
                  i32.ne
                  br_if 2 (;@5;)
                  i32.const 1057840
                  i32.const 1057840
                  i32.load
                  i32.const -2
                  local.get 2
                  i32.const 3
                  i32.shr_u
                  i32.rotl
                  i32.and
                  i32.store
                  br 5 (;@2;)
                end
                local.get 0
                i32.load offset=24
                local.set 6
                local.get 0
                local.get 3
                i32.ne
                if  ;; label = @7
                  local.get 0
                  i32.load offset=8
                  local.tee 2
                  local.get 3
                  i32.store offset=12
                  local.get 3
                  local.get 2
                  i32.store offset=8
                  br 4 (;@3;)
                end
                local.get 0
                i32.load offset=20
                local.tee 4
                if (result i32)  ;; label = @7
                  local.get 0
                  i32.const 20
                  i32.add
                else
                  local.get 0
                  i32.load offset=16
                  local.tee 4
                  i32.eqz
                  br_if 3 (;@4;)
                  local.get 0
                  i32.const 16
                  i32.add
                end
                local.set 2
                loop  ;; label = @7
                  local.get 2
                  local.set 7
                  local.get 4
                  local.tee 3
                  i32.const 20
                  i32.add
                  local.set 2
                  local.get 3
                  i32.load offset=20
                  local.tee 4
                  br_if 0 (;@7;)
                  local.get 3
                  i32.const 16
                  i32.add
                  local.set 2
                  local.get 3
                  i32.load offset=16
                  local.tee 4
                  br_if 0 (;@7;)
                end
                local.get 7
                i32.const 0
                i32.store
                br 3 (;@3;)
              end
              local.get 5
              i32.load offset=4
              local.tee 2
              i32.const 3
              i32.and
              i32.const 3
              i32.ne
              br_if 3 (;@2;)
              local.get 5
              local.get 2
              i32.const -2
              i32.and
              i32.store offset=4
              i32.const 1057848
              local.get 1
              i32.store
              local.get 5
              local.get 1
              i32.store
              local.get 0
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              return
            end
            local.get 3
            local.get 4
            i32.store offset=8
            local.get 4
            local.get 3
            i32.store offset=12
            br 2 (;@2;)
          end
          i32.const 0
          local.set 3
        end
        local.get 6
        i32.eqz
        br_if 0 (;@2;)
        block  ;; label = @3
          local.get 0
          i32.load offset=28
          local.tee 2
          i32.const 2
          i32.shl
          i32.const 1058144
          i32.add
          local.tee 4
          i32.load
          local.get 0
          i32.eq
          if  ;; label = @4
            local.get 4
            local.get 3
            i32.store
            local.get 3
            br_if 1 (;@3;)
            i32.const 1057844
            i32.const 1057844
            i32.load
            i32.const -2
            local.get 2
            i32.rotl
            i32.and
            i32.store
            br 2 (;@2;)
          end
          local.get 6
          i32.const 16
          i32.const 20
          local.get 6
          i32.load offset=16
          local.get 0
          i32.eq
          select
          i32.add
          local.get 3
          i32.store
          local.get 3
          i32.eqz
          br_if 1 (;@2;)
        end
        local.get 3
        local.get 6
        i32.store offset=24
        local.get 0
        i32.load offset=16
        local.tee 2
        if  ;; label = @3
          local.get 3
          local.get 2
          i32.store offset=16
          local.get 2
          local.get 3
          i32.store offset=24
        end
        local.get 0
        i32.load offset=20
        local.tee 2
        i32.eqz
        br_if 0 (;@2;)
        local.get 3
        local.get 2
        i32.store offset=20
        local.get 2
        local.get 3
        i32.store offset=24
      end
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 5
              i32.load offset=4
              local.tee 2
              i32.const 2
              i32.and
              i32.eqz
              if  ;; label = @6
                i32.const 1057864
                i32.load
                local.get 5
                i32.eq
                if  ;; label = @7
                  i32.const 1057864
                  local.get 0
                  i32.store
                  i32.const 1057852
                  i32.const 1057852
                  i32.load
                  local.get 1
                  i32.add
                  local.tee 1
                  i32.store
                  local.get 0
                  local.get 1
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  i32.const 1057860
                  i32.load
                  i32.ne
                  br_if 6 (;@1;)
                  i32.const 1057848
                  i32.const 0
                  i32.store
                  i32.const 1057860
                  i32.const 0
                  i32.store
                  return
                end
                i32.const 1057860
                i32.load
                local.get 5
                i32.eq
                if  ;; label = @7
                  i32.const 1057860
                  local.get 0
                  i32.store
                  i32.const 1057848
                  i32.const 1057848
                  i32.load
                  local.get 1
                  i32.add
                  local.tee 1
                  i32.store
                  local.get 0
                  local.get 1
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 1
                  i32.add
                  local.get 1
                  i32.store
                  return
                end
                local.get 2
                i32.const -8
                i32.and
                local.get 1
                i32.add
                local.set 1
                local.get 5
                i32.load offset=12
                local.set 3
                local.get 2
                i32.const 255
                i32.le_u
                if  ;; label = @7
                  local.get 3
                  local.get 5
                  i32.load offset=8
                  local.tee 4
                  i32.eq
                  if  ;; label = @8
                    i32.const 1057840
                    i32.const 1057840
                    i32.load
                    i32.const -2
                    local.get 2
                    i32.const 3
                    i32.shr_u
                    i32.rotl
                    i32.and
                    i32.store
                    br 5 (;@3;)
                  end
                  local.get 3
                  local.get 4
                  i32.store offset=8
                  local.get 4
                  local.get 3
                  i32.store offset=12
                  br 4 (;@3;)
                end
                local.get 5
                i32.load offset=24
                local.set 6
                local.get 3
                local.get 5
                i32.ne
                if  ;; label = @7
                  local.get 5
                  i32.load offset=8
                  local.tee 2
                  local.get 3
                  i32.store offset=12
                  local.get 3
                  local.get 2
                  i32.store offset=8
                  br 3 (;@4;)
                end
                local.get 5
                i32.load offset=20
                local.tee 4
                if (result i32)  ;; label = @7
                  local.get 5
                  i32.const 20
                  i32.add
                else
                  local.get 5
                  i32.load offset=16
                  local.tee 4
                  i32.eqz
                  br_if 2 (;@5;)
                  local.get 5
                  i32.const 16
                  i32.add
                end
                local.set 2
                loop  ;; label = @7
                  local.get 2
                  local.set 7
                  local.get 4
                  local.tee 3
                  i32.const 20
                  i32.add
                  local.set 2
                  local.get 3
                  i32.load offset=20
                  local.tee 4
                  br_if 0 (;@7;)
                  local.get 3
                  i32.const 16
                  i32.add
                  local.set 2
                  local.get 3
                  i32.load offset=16
                  local.tee 4
                  br_if 0 (;@7;)
                end
                local.get 7
                i32.const 0
                i32.store
                br 2 (;@4;)
              end
              local.get 5
              local.get 2
              i32.const -2
              i32.and
              i32.store offset=4
              local.get 0
              local.get 1
              i32.add
              local.get 1
              i32.store
              local.get 0
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              br 3 (;@2;)
            end
            i32.const 0
            local.set 3
          end
          local.get 6
          i32.eqz
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 5
            i32.load offset=28
            local.tee 2
            i32.const 2
            i32.shl
            i32.const 1058144
            i32.add
            local.tee 4
            i32.load
            local.get 5
            i32.eq
            if  ;; label = @5
              local.get 4
              local.get 3
              i32.store
              local.get 3
              br_if 1 (;@4;)
              i32.const 1057844
              i32.const 1057844
              i32.load
              i32.const -2
              local.get 2
              i32.rotl
              i32.and
              i32.store
              br 2 (;@3;)
            end
            local.get 6
            i32.const 16
            i32.const 20
            local.get 6
            i32.load offset=16
            local.get 5
            i32.eq
            select
            i32.add
            local.get 3
            i32.store
            local.get 3
            i32.eqz
            br_if 1 (;@3;)
          end
          local.get 3
          local.get 6
          i32.store offset=24
          local.get 5
          i32.load offset=16
          local.tee 2
          if  ;; label = @4
            local.get 3
            local.get 2
            i32.store offset=16
            local.get 2
            local.get 3
            i32.store offset=24
          end
          local.get 5
          i32.load offset=20
          local.tee 2
          i32.eqz
          br_if 0 (;@3;)
          local.get 3
          local.get 2
          i32.store offset=20
          local.get 2
          local.get 3
          i32.store offset=24
        end
        local.get 0
        local.get 1
        i32.add
        local.get 1
        i32.store
        local.get 0
        local.get 1
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 0
        i32.const 1057860
        i32.load
        i32.ne
        br_if 0 (;@2;)
        i32.const 1057848
        local.get 1
        i32.store
        return
      end
      local.get 1
      i32.const 255
      i32.le_u
      if  ;; label = @2
        local.get 1
        i32.const -8
        i32.and
        i32.const 1057880
        i32.add
        local.set 2
        block (result i32)  ;; label = @3
          i32.const 1057840
          i32.load
          local.tee 3
          i32.const 1
          local.get 1
          i32.const 3
          i32.shr_u
          i32.shl
          local.tee 1
          i32.and
          i32.eqz
          if  ;; label = @4
            i32.const 1057840
            local.get 1
            local.get 3
            i32.or
            i32.store
            local.get 2
            br 1 (;@3;)
          end
          local.get 2
          i32.load offset=8
        end
        local.tee 1
        local.get 0
        i32.store offset=12
        local.get 2
        local.get 0
        i32.store offset=8
        local.get 0
        local.get 2
        i32.store offset=12
        local.get 0
        local.get 1
        i32.store offset=8
        return
      end
      i32.const 31
      local.set 3
      local.get 1
      i32.const 16777215
      i32.le_u
      if  ;; label = @2
        local.get 1
        i32.const 38
        local.get 1
        i32.const 8
        i32.shr_u
        i32.clz
        local.tee 2
        i32.sub
        i32.shr_u
        i32.const 1
        i32.and
        local.get 2
        i32.const 1
        i32.shl
        i32.sub
        i32.const 62
        i32.add
        local.set 3
      end
      local.get 0
      local.get 3
      i32.store offset=28
      local.get 0
      i64.const 0
      i64.store offset=16 align=4
      local.get 3
      i32.const 2
      i32.shl
      i32.const 1058144
      i32.add
      local.set 2
      i32.const 1057844
      i32.load
      local.tee 4
      i32.const 1
      local.get 3
      i32.shl
      local.tee 7
      i32.and
      i32.eqz
      if  ;; label = @2
        local.get 2
        local.get 0
        i32.store
        i32.const 1057844
        local.get 4
        local.get 7
        i32.or
        i32.store
        local.get 0
        local.get 2
        i32.store offset=24
        local.get 0
        local.get 0
        i32.store offset=8
        local.get 0
        local.get 0
        i32.store offset=12
        return
      end
      local.get 1
      i32.const 25
      local.get 3
      i32.const 1
      i32.shr_u
      i32.sub
      i32.const 0
      local.get 3
      i32.const 31
      i32.ne
      select
      i32.shl
      local.set 3
      local.get 2
      i32.load
      local.set 2
      block  ;; label = @2
        loop  ;; label = @3
          local.get 2
          local.tee 4
          i32.load offset=4
          i32.const -8
          i32.and
          local.get 1
          i32.eq
          br_if 1 (;@2;)
          local.get 3
          i32.const 29
          i32.shr_u
          local.set 2
          local.get 3
          i32.const 1
          i32.shl
          local.set 3
          local.get 4
          local.get 2
          i32.const 4
          i32.and
          i32.add
          i32.const 16
          i32.add
          local.tee 7
          i32.load
          local.tee 2
          br_if 0 (;@3;)
        end
        local.get 7
        local.get 0
        i32.store
        local.get 0
        local.get 4
        i32.store offset=24
        local.get 0
        local.get 0
        i32.store offset=12
        local.get 0
        local.get 0
        i32.store offset=8
        return
      end
      local.get 4
      i32.load offset=8
      local.tee 1
      local.get 0
      i32.store offset=12
      local.get 4
      local.get 0
      i32.store offset=8
      local.get 0
      i32.const 0
      i32.store offset=24
      local.get 0
      local.get 4
      i32.store offset=12
      local.get 0
      local.get 1
      i32.store offset=8
    end)
  (func (;102;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    local.get 1
    i32.const -68
    i32.gt_u
    if  ;; label = @1
      i32.const 48
      return
    end
    block (result i32)  ;; label = @1
      local.get 1
      i32.const -80
      i32.ge_u
      if  ;; label = @2
        i32.const 1058336
        i32.const 48
        i32.store
        i32.const 0
        br 1 (;@1;)
      end
      i32.const 0
      i32.const 16
      local.get 1
      i32.const 19
      i32.add
      i32.const -16
      i32.and
      local.get 1
      i32.const 11
      i32.lt_u
      select
      local.tee 5
      i32.const 28
      i32.add
      call 97
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      drop
      local.get 1
      i32.const 8
      i32.sub
      local.set 2
      block  ;; label = @2
        local.get 1
        i32.const 15
        i32.and
        i32.eqz
        if  ;; label = @3
          local.get 2
          local.set 1
          br 1 (;@2;)
        end
        local.get 1
        i32.const 4
        i32.sub
        local.tee 6
        i32.load
        local.tee 7
        i32.const -8
        i32.and
        i32.const 16
        i32.const 0
        local.get 1
        i32.const 15
        i32.add
        i32.const -16
        i32.and
        i32.const 8
        i32.sub
        local.tee 1
        local.get 2
        i32.sub
        i32.const 15
        i32.le_u
        select
        local.get 1
        i32.add
        local.tee 1
        local.get 2
        i32.sub
        local.tee 3
        i32.sub
        local.set 4
        local.get 7
        i32.const 3
        i32.and
        i32.eqz
        if  ;; label = @3
          local.get 1
          local.get 4
          i32.store offset=4
          local.get 1
          local.get 2
          i32.load
          local.get 3
          i32.add
          i32.store
          br 1 (;@2;)
        end
        local.get 1
        local.get 4
        local.get 1
        i32.load offset=4
        i32.const 1
        i32.and
        i32.or
        i32.const 2
        i32.or
        i32.store offset=4
        local.get 1
        local.get 4
        i32.add
        local.tee 4
        local.get 4
        i32.load offset=4
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 6
        local.get 3
        local.get 6
        i32.load
        i32.const 1
        i32.and
        i32.or
        i32.const 2
        i32.or
        i32.store
        local.get 2
        local.get 3
        i32.add
        local.tee 4
        local.get 4
        i32.load offset=4
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 2
        local.get 3
        call 101
      end
      block  ;; label = @2
        local.get 1
        i32.load offset=4
        local.tee 2
        i32.const 3
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.const -8
        i32.and
        local.tee 3
        local.get 5
        i32.const 16
        i32.add
        i32.le_u
        br_if 0 (;@2;)
        local.get 1
        local.get 5
        local.get 2
        i32.const 1
        i32.and
        i32.or
        i32.const 2
        i32.or
        i32.store offset=4
        local.get 1
        local.get 5
        i32.add
        local.tee 2
        local.get 3
        local.get 5
        i32.sub
        local.tee 5
        i32.const 3
        i32.or
        i32.store offset=4
        local.get 1
        local.get 3
        i32.add
        local.tee 3
        local.get 3
        i32.load offset=4
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 2
        local.get 5
        call 101
      end
      local.get 1
      i32.const 8
      i32.add
    end
    local.tee 1
    i32.eqz
    if  ;; label = @1
      i32.const 48
      return
    end
    local.get 0
    local.get 1
    i32.store
    i32.const 0)
  (func (;103;) (type 2) (param i32)
    local.get 0
    call 23
    unreachable)
  (func (;104;) (type 6) (param i32) (result i32)
    local.get 0
    i32.eqz
    if  ;; label = @1
      memory.size
      i32.const 16
      i32.shl
      return
    end
    block  ;; label = @1
      local.get 0
      i32.const 65535
      i32.and
      br_if 0 (;@1;)
      local.get 0
      i32.const 0
      i32.lt_s
      br_if 0 (;@1;)
      local.get 0
      i32.const 16
      i32.shr_u
      memory.grow
      local.tee 0
      i32.const -1
      i32.eq
      if  ;; label = @2
        i32.const 1058336
        i32.const 48
        i32.store
        i32.const -1
        return
      end
      local.get 0
      i32.const 16
      i32.shl
      return
    end
    unreachable)
  (func (;105;) (type 2) (param i32)
    local.get 0
    call 103
    unreachable)
  (func (;106;) (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 32
        i32.le_u
        if  ;; label = @3
          local.get 1
          i32.const 3
          i32.and
          i32.eqz
          br_if 1 (;@2;)
          local.get 2
          i32.eqz
          br_if 1 (;@2;)
          local.get 0
          local.get 1
          i32.load8_u
          i32.store8
          local.get 2
          i32.const 1
          i32.sub
          local.set 5
          local.get 0
          i32.const 1
          i32.add
          local.set 4
          local.get 1
          i32.const 1
          i32.add
          local.tee 3
          i32.const 3
          i32.and
          i32.eqz
          br_if 2 (;@1;)
          local.get 5
          i32.eqz
          br_if 2 (;@1;)
          local.get 0
          local.get 1
          i32.load8_u offset=1
          i32.store8 offset=1
          local.get 2
          i32.const 2
          i32.sub
          local.set 5
          local.get 0
          i32.const 2
          i32.add
          local.set 4
          local.get 1
          i32.const 2
          i32.add
          local.tee 3
          i32.const 3
          i32.and
          i32.eqz
          br_if 2 (;@1;)
          local.get 5
          i32.eqz
          br_if 2 (;@1;)
          local.get 0
          local.get 1
          i32.load8_u offset=2
          i32.store8 offset=2
          local.get 2
          i32.const 3
          i32.sub
          local.set 5
          local.get 0
          i32.const 3
          i32.add
          local.set 4
          local.get 1
          i32.const 3
          i32.add
          local.tee 3
          i32.const 3
          i32.and
          i32.eqz
          br_if 2 (;@1;)
          local.get 5
          i32.eqz
          br_if 2 (;@1;)
          local.get 0
          local.get 1
          i32.load8_u offset=3
          i32.store8 offset=3
          local.get 2
          i32.const 4
          i32.sub
          local.set 5
          local.get 0
          i32.const 4
          i32.add
          local.set 4
          local.get 1
          i32.const 4
          i32.add
          local.set 3
          br 2 (;@1;)
        end
        local.get 0
        local.get 1
        local.get 2
        memory.copy
        local.get 0
        return
      end
      local.get 2
      local.set 5
      local.get 0
      local.set 4
      local.get 1
      local.set 3
    end
    block  ;; label = @1
      local.get 4
      i32.const 3
      i32.and
      local.tee 2
      i32.eqz
      if  ;; label = @2
        block  ;; label = @3
          local.get 5
          i32.const 16
          i32.lt_u
          if  ;; label = @4
            local.get 5
            local.set 2
            br 1 (;@3;)
          end
          local.get 5
          i32.const 16
          i32.sub
          local.tee 2
          i32.const 16
          i32.and
          i32.eqz
          if  ;; label = @4
            local.get 4
            local.get 3
            i64.load align=4
            i64.store align=4
            local.get 4
            local.get 3
            i64.load offset=8 align=4
            i64.store offset=8 align=4
            local.get 4
            i32.const 16
            i32.add
            local.set 4
            local.get 3
            i32.const 16
            i32.add
            local.set 3
            local.get 2
            local.set 5
          end
          local.get 2
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 5
          local.set 2
          loop  ;; label = @4
            local.get 4
            local.get 3
            i64.load align=4
            i64.store align=4
            local.get 4
            local.get 3
            i64.load offset=8 align=4
            i64.store offset=8 align=4
            local.get 4
            local.get 3
            i64.load offset=16 align=4
            i64.store offset=16 align=4
            local.get 4
            local.get 3
            i64.load offset=24 align=4
            i64.store offset=24 align=4
            local.get 4
            i32.const 32
            i32.add
            local.set 4
            local.get 3
            i32.const 32
            i32.add
            local.set 3
            local.get 2
            i32.const 32
            i32.sub
            local.tee 2
            i32.const 15
            i32.gt_u
            br_if 0 (;@4;)
          end
        end
        local.get 2
        i32.const 8
        i32.ge_u
        if  ;; label = @3
          local.get 4
          local.get 3
          i64.load align=4
          i64.store align=4
          local.get 4
          i32.const 8
          i32.add
          local.set 4
          local.get 3
          i32.const 8
          i32.add
          local.set 3
        end
        local.get 2
        i32.const 4
        i32.and
        if  ;; label = @3
          local.get 4
          local.get 3
          i32.load
          i32.store
          local.get 4
          i32.const 4
          i32.add
          local.set 4
          local.get 3
          i32.const 4
          i32.add
          local.set 3
        end
        local.get 2
        i32.const 2
        i32.and
        if  ;; label = @3
          local.get 4
          local.get 3
          i32.load16_u align=1
          i32.store16 align=1
          local.get 4
          i32.const 2
          i32.add
          local.set 4
          local.get 3
          i32.const 2
          i32.add
          local.set 3
        end
        local.get 2
        i32.const 1
        i32.and
        i32.eqz
        br_if 1 (;@1;)
        local.get 4
        local.get 3
        i32.load8_u
        i32.store8
        local.get 0
        return
      end
      block  ;; label = @2
        block  ;; label = @3
          block (result i32)  ;; label = @4
            block  ;; label = @5
              local.get 5
              i32.const 32
              i32.ge_u
              if  ;; label = @6
                local.get 4
                local.get 3
                i32.load
                local.tee 1
                i32.store8
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 2
                    i32.const 2
                    i32.sub
                    br_table 0 (;@8;) 1 (;@7;) 3 (;@5;)
                  end
                  local.get 4
                  local.get 1
                  i32.const 8
                  i32.shr_u
                  i32.store8 offset=1
                  local.get 4
                  local.get 3
                  i32.const 6
                  i32.add
                  i64.load align=2
                  i64.store offset=6 align=4
                  local.get 4
                  local.get 3
                  i32.load offset=4
                  i32.const 16
                  i32.shl
                  local.get 1
                  i32.const 16
                  i32.shr_u
                  i32.or
                  i32.store offset=2
                  local.get 3
                  i32.const 18
                  i32.add
                  local.set 1
                  i32.const 14
                  local.set 6
                  local.get 3
                  i32.const 14
                  i32.add
                  i32.load align=2
                  local.set 3
                  i32.const 14
                  local.set 5
                  local.get 4
                  i32.const 18
                  i32.add
                  br 3 (;@4;)
                end
                local.get 4
                local.get 3
                i32.const 5
                i32.add
                i64.load align=1
                i64.store offset=5 align=4
                local.get 4
                local.get 3
                i32.load offset=4
                i32.const 24
                i32.shl
                local.get 1
                i32.const 8
                i32.shr_u
                i32.or
                i32.store offset=1
                local.get 3
                i32.const 17
                i32.add
                local.set 1
                i32.const 13
                local.set 6
                local.get 3
                i32.const 13
                i32.add
                i32.load align=1
                local.set 3
                i32.const 15
                local.set 5
                local.get 4
                i32.const 17
                i32.add
                br 2 (;@4;)
              end
              block (result i32)  ;; label = @6
                local.get 5
                i32.const 16
                i32.lt_u
                if  ;; label = @7
                  local.get 4
                  local.set 2
                  local.get 3
                  br 1 (;@6;)
                end
                local.get 4
                local.get 3
                i32.load8_u
                i32.store8
                local.get 4
                local.get 3
                i32.load offset=1 align=1
                i32.store offset=1 align=1
                local.get 4
                local.get 3
                i64.load offset=5 align=1
                i64.store offset=5 align=1
                local.get 4
                local.get 3
                i32.load16_u offset=13 align=1
                i32.store16 offset=13 align=1
                local.get 4
                local.get 3
                i32.load8_u offset=15
                i32.store8 offset=15
                local.get 4
                i32.const 16
                i32.add
                local.set 2
                local.get 3
                i32.const 16
                i32.add
              end
              local.set 1
              local.get 5
              i32.const 8
              i32.and
              br_if 2 (;@3;)
              br 3 (;@2;)
            end
            local.get 4
            local.get 1
            i32.const 16
            i32.shr_u
            i32.store8 offset=2
            local.get 4
            local.get 1
            i32.const 8
            i32.shr_u
            i32.store8 offset=1
            local.get 4
            local.get 3
            i32.const 7
            i32.add
            i64.load align=1
            i64.store offset=7 align=4
            local.get 4
            local.get 3
            i32.load offset=4
            i32.const 8
            i32.shl
            local.get 1
            i32.const 24
            i32.shr_u
            i32.or
            i32.store offset=3
            local.get 3
            i32.const 19
            i32.add
            local.set 1
            i32.const 15
            local.set 6
            local.get 3
            i32.const 15
            i32.add
            i32.load align=1
            local.set 3
            i32.const 13
            local.set 5
            local.get 4
            i32.const 19
            i32.add
          end
          local.set 2
          local.get 4
          local.get 6
          i32.add
          local.get 3
          i32.store
        end
        local.get 2
        local.get 1
        i64.load align=1
        i64.store align=1
        local.get 2
        i32.const 8
        i32.add
        local.set 2
        local.get 1
        i32.const 8
        i32.add
        local.set 1
      end
      local.get 5
      i32.const 4
      i32.and
      if  ;; label = @2
        local.get 2
        local.get 1
        i32.load align=1
        i32.store align=1
        local.get 2
        i32.const 4
        i32.add
        local.set 2
        local.get 1
        i32.const 4
        i32.add
        local.set 1
      end
      local.get 5
      i32.const 2
      i32.and
      if  ;; label = @2
        local.get 2
        local.get 1
        i32.load16_u align=1
        i32.store16 align=1
        local.get 2
        i32.const 2
        i32.add
        local.set 2
        local.get 1
        i32.const 2
        i32.add
        local.set 1
      end
      local.get 5
      i32.const 1
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 1
      i32.load8_u
      i32.store8
    end
    local.get 0)
  (func (;107;) (type 6) (param i32) (result i32)
    (local i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        local.tee 1
        i32.const 3
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load8_u
        i32.eqz
        if  ;; label = @3
          i32.const 0
          return
        end
        local.get 0
        i32.const 1
        i32.add
        local.tee 1
        i32.const 3
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load8_u
        i32.eqz
        br_if 1 (;@1;)
        local.get 0
        i32.const 2
        i32.add
        local.tee 1
        i32.const 3
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load8_u
        i32.eqz
        br_if 1 (;@1;)
        local.get 0
        i32.const 3
        i32.add
        local.tee 1
        i32.const 3
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load8_u
        i32.eqz
        br_if 1 (;@1;)
        local.get 0
        i32.const 4
        i32.add
        local.tee 1
        i32.const 3
        i32.and
        br_if 1 (;@1;)
      end
      local.get 1
      i32.const 4
      i32.sub
      local.set 2
      local.get 1
      i32.const 5
      i32.sub
      local.set 1
      loop  ;; label = @2
        local.get 1
        i32.const 4
        i32.add
        local.set 1
        local.get 2
        i32.const 4
        i32.add
        local.tee 2
        i32.load
        local.tee 3
        i32.const 16843008
        local.get 3
        i32.sub
        i32.or
        i32.const -2139062144
        i32.and
        i32.const -2139062144
        i32.eq
        br_if 0 (;@2;)
      end
      loop  ;; label = @2
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 2
        i32.load8_u
        local.get 2
        i32.const 1
        i32.add
        local.set 2
        br_if 0 (;@2;)
      end
    end
    local.get 1
    local.get 0
    i32.sub)
  (table (;0;) 44 44 funcref)
  (memory (;0;) 17)
  (global (;0;) (mut i32) (i32.const 1048576))
  (export "memory" (memory 0))
  (export "_start" (func 24))
  (export "__main_void" (func 35))
  (elem (;0;) (i32.const 1) func 40 26 45 46 47 49 64 69 50 51 52 53 54 68 70 71 72 73 80 81 82 53 94 95 96 74 75 76 77 78 79 62 93 83 84 85 86 87 88 89 90 91 92)
  (data (;0;) (i32.const 1048576) "MT_NODE_V1BL_BUCKET_V1ADDR_V2NOTE_V2capacity overflow\00\00\00$\00\10\00\11\00\00\00)BorrowMutErroralready borrowed: \00\00\00O\00\10\00\12\00\00\00[called `Option::unwrap()` on a `None` valueindex out of bounds: the len is  but the index is \00\00\98\00\10\00 \00\00\00\b8\00\10\00\12\00\00\00==assertion `left  right` failed\0a  left: \0a right: \00\00\de\00\10\00\10\00\00\00\ee\00\10\00\17\00\00\00\05\01\10\00\09\00\00\00 right` failed: \0a  left: \00\00\00\de\00\10\00\10\00\00\00(\01\10\00\10\00\00\008\01\10\00\09\00\00\00\05\01\10\00\09\00\00\00: \00\00\01\00\00\00\00\00\00\00d\01\10\00\02\00\00\00\00\00\00\00\0c\00\00\00\04\00\00\00\09\00\00\00\0a\00\00\00\0b\00\00\00    , ,\0a((\0a]0x00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899falsetruerange start index  out of range for slice of length \00o\02\10\00\12\00\00\00\81\02\10\00\22\00\00\00range end index \b4\02\10\00\10\00\00\00\81\02\10\00\22\00\00\00slice index starts at  but ends at \00\d4\02\10\00\16\00\00\00\ea\02\10\00\0d\00\00\00NulError/Users/guillevalin/Documents/dcSpark/ligero-vm/ligero-prover/sdk/rust/src/api.rs\10\03\10\00P\00\00\00\5c\00\00\001\00\00\00Invalid argument length for i64!\10\03\10\00P\00\00\00\89\00\00\00%\00\00\00\10\03\10\00P\00\00\00\8a\00\00\00*\00\00\00\10\03\10\00P\00\00\00\8d\00\00\00)\00\00\00\10\03\10\00P\00\00\00\91\00\00\00,\00\00\00\10\03\10\00P\00\00\00\96\00\00\00#\00\00\00\10\03\10\00P\00\00\00\98\00\00\00+\00\00\00\0c\00\00\00\10\00\00\00\04\00\00\00\0d\00\00\00Error parsing numeric string/Users/guillevalin/Documents/dcSpark/ligero-vm/ligero-prover/sdk/rust/src/bn254fr.rs\1c\04\10\00T\00\00\00\ba\00\00\00%\00\00\000x09c46e9ec68e9bd4fe1faaba294cba38a71aa177534cdd1b6c7dc0dbd0abd7a70x0c0356530896eec42a97ed937f3135cfc5142b3ae405b8343c1d83ffa604cb810x1e28a1d935698ad1142e51182bb54cf4a00ea5aabd6268bd317ea977cc154a300x27af2d831a9d2748080965db30e298e40e5757c3e008db964cf9e2b12b91251f0x1e6f11ce60fc8f513a6a3cfe16ae175a41291462f214cd0879aaf43545b74e030x2a67384d3bbd5e438541819cb681f0be04462ed14c3613d8f719206268d142d30x0b66fdf356093a611609f8e12fbfecf0b985e381f025188936408f5d5c9f45d00x012ee3ec1e78d470830c61093c2ade370b26c83cc5cebeeddaa6852dbdb09e210x0252ba5f6760bfbdfd88f67f8175e3fd6cd1c431b099b6bb2d108e7b445bb1b90x00000000000000000000000000000000000000000000000000000000000000000x179474cceca5ff676c6bec3cef54296354391a8935ff71d6ef5aeaad7ca932f10x2c24261379a51bfa9228ff4a503fd4ed9c1f974a264969b37e1a2589bbed2b910x1cc1d7b62692e63eac2f288bd0695b43c2f63f5001fc0fc553e66c0551801b050x255059301aada98bb2ed55f852979e9600784dbf17fbacd05d9eff5fd9c91b560x28437be3ac1cb2e479e1f5c0eccd32b3aea24234970a8193b11c29ce7e59efd90x28216a442f2e1f711ca4fa6b53766eb118548da8fb4f78d4338762c37f5f20430x2c1f47cd17fa5adf1f39f4e7056dd03feee1efce03094581131f2377323482c90x07abad02b7a5ebc48632bcc9356ceb7dd9dafca276638a63646b8566a621afc90x0230264601ffdf29275b33ffaab51dfe9429f90880a69cd137da0c4d15f96c3c0x1bc973054e51d905a0f168656497ca40a864414557ee289e717e5d66899aa0a90x2e1c22f964435008206c3157e86341edd249aff5c2d8421f2a6b22288f0a67fc0x1224f38df67c5378121c1d5f461bbc509e8ea1598e46c9f7a70452bc2bba86b80x02e4e69d8ba59e519280b4bd9ed0068fd7bfe8cd9dfeda1969d2989186cde20e0x1f1eccc34aaba0137f5df81fc04ff3ee4f19ee364e653f076d47e9735d98018e0x1672ad3d709a353974266c3039a9a7311424448032cd1819eacb8a4d4284f5820x283e3fdc2c6e420c56f44af5192b4ae9cda6961f284d24991d2ed602df8c8fc70x1c2a3d120c550ecfd0db0957170fa013683751f8fdff59d6614fbd69ff394bcc0x216f84877aac6172f7897a7323456efe143a9a43773ea6f296cb6b8177653fbd0x2c0d272becf2a75764ba7e8e3e28d12bceaa47ea61ca59a411a1f51552f947880x16e34299865c0e28484ee7a74c454e9f170a5480abe0508fcb4a6c3d89546f430x175ceba599e96f5b375a232a6fb9cc71772047765802290f48cd939755488fc50x0c7594440dc48c16fead9e1758b028066aa410bfbc354f54d8c5ffbb44a1ee320x1a3c29bc39f21bb5c466db7d7eb6fd8f760e20013ccf912c92479882d919fd8d0x0ccfdd906f3426e5c0986ea049b253400855d349074f5a6695c8eeabcd22e68f0x14f6bc81d9f186f62bdb475ce6c9411866a7a8a3fd065b3ce0e699b67dd9e7960x0962b82789fb3d129702ca70b2f6c5aacc099810c9c495c888edeb7386b970520x1a880af7074d18b3bf20c79de25127bc13284ab01ef02575afef0c8f6a31a86d0x10cba18419a6a332cd5e77f0211c154b20af2924fc20ff3f4c3012bb7ae9311b0x057e62a9a8f89b3ebdc76ba63a9eaca8fa27b7319cae3406756a2849f302f10d0x287c971de91dc0abd44adf5384b4988cb961303bbf65cff5afa0413b44280cee0x21df3388af1687bbb3bca9da0cca908f1e562bc46d4aba4e6f7f7960e306891d0x1be5c887d25bce703e25cc974d0934cd789df8f70b498fd83eff8b560e1682b30x268da36f76e568fb68117175cea2cd0dd2cb5d42fda5acea48d59c2706a0d5c10x0e17ab091f6eae50c609beaf5510ececc5d8bb74135ebd05bd06460cc26a5ed60x04d727e728ffa0a67aee535ab074a43091ef62d8cf83d270040f5caa1f62af400x0ddbd7bf9c29341581b549762bc022ed33702ac10f1bfd862b15417d7e39ca6e0x2790eb3351621752768162e82989c6c234f5b0d1d3af9b588a29c49c8789654b0x1e457c601a63b73e4471950193d8a570395f3d9ab8b2fd0984b764206142f9e90x21ae64301dca9625638d6ab2bbe7135ffa90ecd0c43ff91fc4c686fc46e091b00x0379f63c8ce3468d4da293166f494928854be9e3432e09555858534eed8d350b0x002d56420359d0266a744a080809e054ca0e4921a46686ac8c9f58a324c350490x123158e5965b5d9b1d68b3cd32e10bbeda8d62459e21f4090fc2c5af963515a60x0be29fc40847a941661d14bbf6cbe0420fbb2b6f52836d4e60c80eb49cad9ec10x1ac96991dec2bb0557716142015a453c36db9d859cad5f9a233802f24fdf4c1a0x1596443f763dbcc25f4964fc61d23b3e5e12c9fa97f18a9251ca3355bcb0627e0x12e0bcd3654bdfa76b2861d4ec3aeae0f1857d9f17e715aed6d049eae3ba32120x0fc92b4f1bbea82b9ea73d4af9af2a50ceabac7f37154b1904e6c76c7cf964ba0x1f9c0b1610446442d6f2e592a8013f40b14f7c7722236f4f9c7e9652338727620x0ebd74244ae72675f8cde06157a782f4050d914da38b4c058d159f643dbbf4d30x2cb7f0ed39e16e9f69a9fafd4ab951c03b0671e97346ee397a839839dccfc6d10x1a9d6e2ecff022cc5605443ee41bab20ce761d0514ce526690c72bca7352d9bf0x2a115439607f335a5ea83c3bc44a9331d0c13326a9a7ba3087da182d648ec72f0x23f9b6529b5d040d15b8fa7aee3e3410e738b56305cd44f29535c115c5a4c0600x05872c16db0f72a2249ac6ba484bb9c3a3ce97c16d58b68b260eb939f0e6e8a70x1300bdee08bb7824ca20fb80118075f40219b6151d55b5c52b624a7cdeddf6a70x19b9b63d2f108e17e63817863a8f6c288d7ad29916d98cb1072e4e7b7d52b3760x015bee1357e3c015b5bda237668522f613d1c88726b5ec4224a20128481b4f7f0x2953736e94bb6b9f1b9707a4f1615e4efe1e1ce4bab218cbea92c785b128ffd10x0b069353ba091618862f806180c0385f851b98d372b45f544ce7266ed6608dfc0x304f74d461ccc13115e4e0bcfb93817e55aeb7eb9306b64e4f588ac97d81f4290x15bbf146ce9bca09e8a33f5e77dfe4f5aad2a164a4617a4cb8ee5415cde913fc0x0ab4dfe0c2742cde44901031487964ed9b8f4b850405c10ca9ff23859572c8c60x0e32db320a044e3197f45f7649a19675ef5eedfea546dea9251de39f9639779a\00\00\80\04\10\00B\00\00\00\c2\04\10\00B\00\00\00\04\05\10\00B\00\00\00F\05\10\00B\00\00\00\88\05\10\00B\00\00\00\ca\05\10\00B\00\00\00\0c\06\10\00B\00\00\00N\06\10\00B\00\00\00\90\06\10\00B\00\00\00\d2\06\10\00B\00\00\00\14\07\10\00B\00\00\00\d2\06\10\00B\00\00\00V\07\10\00B\00\00\00\d2\06\10\00B\00\00\00\98\07\10\00B\00\00\00\d2\06\10\00B\00\00\00\da\07\10\00B\00\00\00\d2\06\10\00B\00\00\00\1c\08\10\00B\00\00\00\d2\06\10\00B\00\00\00^\08\10\00B\00\00\00\d2\06\10\00B\00\00\00\a0\08\10\00B\00\00\00\d2\06\10\00B\00\00\00\e2\08\10\00B\00\00\00\d2\06\10\00B\00\00\00$\09\10\00B\00\00\00\d2\06\10\00B\00\00\00f\09\10\00B\00\00\00\d2\06\10\00B\00\00\00\a8\09\10\00B\00\00\00\d2\06\10\00B\00\00\00\ea\09\10\00B\00\00\00\d2\06\10\00B\00\00\00,\0a\10\00B\00\00\00\d2\06\10\00B\00\00\00n\0a\10\00B\00\00\00\d2\06\10\00B\00\00\00\b0\0a\10\00B\00\00\00\d2\06\10\00B\00\00\00\f2\0a\10\00B\00\00\00\d2\06\10\00B\00\00\004\0b\10\00B\00\00\00\d2\06\10\00B\00\00\00v\0b\10\00B\00\00\00\d2\06\10\00B\00\00\00\b8\0b\10\00B\00\00\00\d2\06\10\00B\00\00\00\fa\0b\10\00B\00\00\00\d2\06\10\00B\00\00\00<\0c\10\00B\00\00\00\d2\06\10\00B\00\00\00~\0c\10\00B\00\00\00\d2\06\10\00B\00\00\00\c0\0c\10\00B\00\00\00\d2\06\10\00B\00\00\00\02\0d\10\00B\00\00\00\d2\06\10\00B\00\00\00D\0d\10\00B\00\00\00\d2\06\10\00B\00\00\00\86\0d\10\00B\00\00\00\d2\06\10\00B\00\00\00\c8\0d\10\00B\00\00\00\d2\06\10\00B\00\00\00\0a\0e\10\00B\00\00\00\d2\06\10\00B\00\00\00L\0e\10\00B\00\00\00\d2\06\10\00B\00\00\00\8e\0e\10\00B\00\00\00\d2\06\10\00B\00\00\00\d0\0e\10\00B\00\00\00\d2\06\10\00B\00\00\00\12\0f\10\00B\00\00\00\d2\06\10\00B\00\00\00T\0f\10\00B\00\00\00\d2\06\10\00B\00\00\00\96\0f\10\00B\00\00\00\d2\06\10\00B\00\00\00\d8\0f\10\00B\00\00\00\d2\06\10\00B\00\00\00\1a\10\10\00B\00\00\00\d2\06\10\00B\00\00\00\5c\10\10\00B\00\00\00\d2\06\10\00B\00\00\00\9e\10\10\00B\00\00\00\d2\06\10\00B\00\00\00\e0\10\10\00B\00\00\00\d2\06\10\00B\00\00\00\22\11\10\00B\00\00\00\d2\06\10\00B\00\00\00d\11\10\00B\00\00\00\d2\06\10\00B\00\00\00\a6\11\10\00B\00\00\00\d2\06\10\00B\00\00\00\e8\11\10\00B\00\00\00\d2\06\10\00B\00\00\00*\12\10\00B\00\00\00\d2\06\10\00B\00\00\00l\12\10\00B\00\00\00\d2\06\10\00B\00\00\00\ae\12\10\00B\00\00\00\d2\06\10\00B\00\00\00\f0\12\10\00B\00\00\00\d2\06\10\00B\00\00\002\13\10\00B\00\00\00\d2\06\10\00B\00\00\00t\13\10\00B\00\00\00\d2\06\10\00B\00\00\00\b6\13\10\00B\00\00\00\d2\06\10\00B\00\00\00\f8\13\10\00B\00\00\00\d2\06\10\00B\00\00\00:\14\10\00B\00\00\00\d2\06\10\00B\00\00\00|\14\10\00B\00\00\00\d2\06\10\00B\00\00\00\be\14\10\00B\00\00\00\d2\06\10\00B\00\00\00\00\15\10\00B\00\00\00\d2\06\10\00B\00\00\00B\15\10\00B\00\00\00\84\15\10\00B\00\00\00\c6\15\10\00B\00\00\00\08\16\10\00B\00\00\00J\16\10\00B\00\00\00\8c\16\10\00B\00\00\00\ce\16\10\00B\00\00\00\10\17\10\00B\00\00\00/Users/guillevalin/Documents/dcSpark/ligero-vm/ligero-prover/sdk/rust/src/poseidon2.rs\00\00T\1b\10\00V\00\00\00v\00\00\00\18\00\00\00T\1b\10\00V\00\00\00z\00\00\005\00\00\00T\1b\10\00V\00\00\00l\00\00\00\1e\00\00\00T\1b\10\00V\00\00\00\84\00\00\00\14\00\00\00T\1b\10\00V\00\00\00\8c\00\00\00-\00\00\00T\1b\10\00V\00\00\00\88\00\00\00\18\00\00\00T\1b\10\00V\00\00\00\b5\00\00\00.\00\00\00T\1b\10\00V\00\00\00\b6\00\00\00.\00\00\00T\1b\10\00V\00\00\00\bb\00\00\00.\00\00\00library/std/src/panicking.rs/\00\00\00\00\00\00\00\04\00\00\00\04\00\00\00\0e\00\00\00/rustc/6b00bc3880198600130e1cf62b8f8a93494488cc/library/alloc/src/raw_vec/mod.rsl\1c\10\00P\00\00\00.\02\00\00\11\00\00\00:\00\00\00\01\00\00\00\00\00\00\00\cc\1c\10\00\01\00\00\00\cc\1c\10\00\01\00\00\00\0f\00\00\00\0c\00\00\00\04\00\00\00\10\00\00\00\11\00\00\00\12\00\00\00\0f\00\00\00\0c\00\00\00\04\00\00\00\13\00\00\00\14\00\00\00\15\00\00\00\16\00\00\00\0c\00\00\00\04\00\00\00\17\00\00\00\18\00\00\00\19\00\00\00/rustc/6b00bc3880198600130e1cf62b8f8a93494488cc/library/alloc/src/slice.rs\00\000\1d\10\00J\00\00\00\be\01\00\00\1d\00\00\00library/std/src/rt.rs\00\00\00\8c\1d\10\00\15\00\00\00\86\00\00\00\0d\00\00\00library/std/src/thread/mod.rsfailed to generate unique thread ID: bitspace exhausted\d1\1d\10\007\00\00\00\b4\1d\10\00\1d\00\00\00\a9\04\00\00\0d\00\00\00mainRUST_BACKTRACE\00\00\01\00\00\00\00\00\00\00failed to write whole buffer<\1e\10\00\1c\00\00\00\17\00\00\00\00\00\00\00\02\00\00\00X\1e\10\00library/std/src/io/stdio.rs\00p\1e\10\00\1b\00\00\00\e3\02\00\00\13\00\00\00library/std/src/io/mod.rsa formatting trait implementation returned an error when the underlying stream did not\00\b5\1e\10\00V\00\00\00\9c\1e\10\00\19\00\00\00\88\02\00\00\11\00\00\00\9c\1e\10\00\19\00\00\00\08\06\00\00 \00\00\00advancing io slices beyond their length\004\1f\10\00'\00\00\00\9c\1e\10\00\19\00\00\00\0a\06\00\00\0d\00\00\00advancing IoSlice beyond its length\00t\1f\10\00#\00\00\00library/std/src/sys/io/io_slice/wasi.rs\00\a0\1f\10\00'\00\00\00\14\00\00\00\0d\00\00\00\9c\1e\10\00\19\00\00\00\09\07\00\00$\00\00\00panicked at :\0acannot recursively acquire mutex\00\00\f6\1f\10\00 \00\00\00library/std/src/sys/sync/mutex/no_threads.rs  \10\00,\00\00\00\13\00\00\00\09\00\00\00library/std/src/sync/poison/once.rs\00\5c \10\00#\00\00\00\9b\00\00\002\00\00\00stack backtrace:\0anote: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.\0amemory allocation of  bytes failed\0a\f9 \10\00\15\00\00\00\0e!\10\00\0e")
  (data (;1;) (i32.const 1057076) "\01\00\00\00\1a\00\00\00\1b\00\00\00\1c\00\00\00\1d\00\00\00\1e\00\00\00\1f\00\00\00 \00\00\00note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\0a\00\00T!\10\00N\00\00\00<unnamed>\00\00\00<\1c\10\00\1c\00\00\00\1d\01\00\00.\00\00\00\0athread '' panicked at \0a\c8!\10\00\09\00\00\00\d1!\10\00\0e\00\00\00\f4\1f\10\00\02\00\00\00\df!\10\00\01\00\00\00\16\00\00\00\0c\00\00\00\04\00\00\00!\00\00\00\00\00\00\00\08\00\00\00\04\00\00\00\22\00\00\00\00\00\00\00\08\00\00\00\04\00\00\00#\00\00\00$\00\00\00%\00\00\00&\00\00\00'\00\00\00\10\00\00\00\04\00\00\00(\00\00\00)\00\00\00*\00\00\00+\00\00\00Box<dyn Any>aborting due to panic at \00\00\00d\22\10\00\19\00\00\00\f4\1f\10\00\02\00\00\00\df!\10\00\01\00\00\00\0athread panicked while processing panic. aborting.\0a\00\e8\1f\10\00\0c\00\00\00\f4\1f\10\00\02\00\00\00\98\22\10\003\00\00\00thread caused non-unwinding panic. aborting.\0a\00\00\00\e4\22\10\00-\00\00\00Once instance has previously been poisoned\00\00\1c#\10\00*\00\00\00one-time initialization may not be performed recursivelyP#\10\008\00\00\00fatal runtime error: rwlock locked for writing, aborting\0a\00\00\00\90#\10\009")
  (data (;2;) (i32.const 1057748) "\01\00\00\00X\1c\10\00\ff\ff\ff\ff"))
