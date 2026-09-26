#!/bin/bash
# karatOS Build System - Size / Footprint Module (Milestone 3)
# Enforces the 64 kB ROM/SRAM memory-budget contract:
#   - ROM  footprint = .text + .rodata + .vector_table (+ .data load image)
#   - RAM  footprint = .data + .bss + .heap + .stack
# Budgets and per-section limits are declared in build/configs/global.toml
# under [memory_budget], so adding a new core never touches this module.

# ---------------------------------------------------------------------------
# Configuration helpers (values come from global.toml, with safe defaults)
# ---------------------------------------------------------------------------

# Resolve the configured budget for a target.
# Sets: BUDGET_ROM_BYTES, BUDGET_RAM_BYTES, BUDGET_WARN_PCT
load_size_budget() {
    local target="$1"

    local rom ram warn
    rom=$(load_toml_value "$CONFIG_DIR/global.toml" "memory_budget.${target}.max_rom_bytes" "")
    ram=$(load_toml_value "$CONFIG_DIR/global.toml" "memory_budget.${target}.max_ram_bytes" "")
    warn=$(load_toml_value "$CONFIG_DIR/global.toml" "memory_budget.warn_threshold_pct" "90")

    # Fall back to the generic default entry if no target-specific override
    if [[ -z "$rom" ]]; then
        rom=$(load_toml_value "$CONFIG_DIR/global.toml" "memory_budget.default.max_rom_bytes" "65536")
    fi
    if [[ -z "$ram" ]]; then
        ram=$(load_toml_value "$CONFIG_DIR/global.toml" "memory_budget.default.max_ram_bytes" "65536")
    fi

    BUDGET_ROM_BYTES="$rom"
    BUDGET_RAM_BYTES="$ram"
    BUDGET_WARN_PCT="$warn"

    log_debug "Size budget for '$target': ROM=${BUDGET_ROM_BYTES}B RAM=${BUDGET_RAM_BYTES}B warn@${BUDGET_WARN_PCT}%"
}

# ---------------------------------------------------------------------------
# ELF section introspection (uses binutils readelf/size; no rust toolchain
# required at check time, so it also works on pre-built artifacts in CI)
# ---------------------------------------------------------------------------

# Locate an ELF tool. Honors READELF/SIZE_BIN overrides for cross environments.
_find_elf_tool() {
    local tool="$1"
    if command -v "$tool" >/dev/null 2>&1; then
        echo "$tool"
    elif command -v "llvm-$tool" >/dev/null 2>&1; then
        echo "llvm-$tool"
    else
        return 1
    fi
}

# Get the size of one ELF section (bytes). Prints 0 if the section is absent.
# Portable: no gawk-only builtins; matches on the whole line so it works with
# mawk/busybox awk regardless of how readelf brackets the section index.
get_section_size() {
    local elf="$1" section="$2"
    local readelf_bin
    readelf_bin=$(_find_elf_tool readelf) || { warning "readelf not found"; echo 0; return 1; }

    "$readelf_bin" -S -W "$elf" 2>/dev/null \
        | awk -v sec="$section" '
            function hex2dec(h,   i, c, v) {
                v = 0
                h = tolower(h)
                for (i = 1; i <= length(h); i++) {
                    c = substr(h, i, 1)
                    if (c >= "0" && c <= "9")      v = v * 16 + (c + 0)
                    else if (c == "a")             v = v * 16 + 10
                    else if (c == "b")             v = v * 16 + 11
                    else if (c == "c")             v = v * 16 + 12
                    else if (c == "d")             v = v * 16 + 13
                    else if (c == "e")             v = v * 16 + 14
                    else if (c == "f")             v = v * 16 + 15
                }
                return v
            }
            # Section rows start with the bracketed index; because readelf pads
            # "[ 1]" with a space, awk splits it into two fields ("[" and "1]"),
            # so match on $1 == "[" instead of an anchored regex.
            # Layout once past the index: NAME TYPE ADDR OFF SIZE ...
            $1 == "[" {
                for (i = 2; i <= NF; i++) {
                    if ($i == sec) { print hex2dec($(i + 4)); found = 1; exit }
                }
            }
            END { if (!found) print 0 }'
}

# Sum of ROM-resident sections (what must fit in Flash/ROM).
compute_rom_footprint() {
    local elf="$1"
    local total=0
    for sec in .vector_table .text .rodata .ARM.exidx .data; do
        local s
        s=$(get_section_size "$elf" "$sec")
        total=$((total + s))
    done
    echo "$total"
}

# Sum of RAM-resident sections (what must fit in SRAM at runtime).
compute_ram_footprint() {
    local elf="$1"
    local total=0
    for sec in .data .bss .heap .stack; do
        local s
        s=$(get_section_size "$elf" "$sec")
        total=$((total + s))
    done
    echo "$total"
}

# ---------------------------------------------------------------------------
# Report / enforcement
# ---------------------------------------------------------------------------

# Human-readable footprint report for one built binary.
report_footprint() {
    local target="$1" build_type="$2"
    local triple
    triple=$(get_target_triple "$target")
    local elf="$BUILD_ROOT/target/$triple/$build_type/kernel"

    if [[ ! -f "$elf" ]]; then
        error "Footprint report: binary not found: $elf"
    fi

    load_size_budget "$target"

    local rom ram
    rom=$(compute_rom_footprint "$elf")
    ram=$(compute_ram_footprint "$elf")

    local rom_pct ram_pct
    rom_pct=$((rom * 100 / BUDGET_ROM_BYTES))
    ram_pct=$((ram * 100 / BUDGET_RAM_BYTES))

    echo ""
    echo "=== Memory Footprint: $target ($build_type) ==="
    printf "%-16s %10s %10s %8s %6s\n" "SECTION" "BYTES" "LIMIT" "USED" "STATUS"
    printf "%-16s %10d %10d %7d%% %6s\n" "ROM (.text+.rodata+...)" "$rom" "$BUDGET_ROM_BYTES" "$rom_pct" "$(budget_status "$rom" "$BUDGET_ROM_BYTES")"
    printf "%-16s %10d %10d %7d%% %6s\n" "RAM (.data+.bss+...)" "$ram" "$BUDGET_RAM_BYTES" "$ram_pct" "$(budget_status "$ram" "$BUDGET_RAM_BYTES")"

    # Per-section detail (helps spot regressions like .data bloat)
    echo ""
    echo "Section detail:"
    for sec in .vector_table .text .rodata .data .bss .heap .stack; do
        local s
        s=$(get_section_size "$elf" "$sec")
        printf "  %-16s %10d bytes\n" "$sec" "$s"
    done

    ROM_FOOTPRINT="$rom"
    RAM_FOOTPRINT="$ram"
}

# PASS / WARN / FAIL string for value vs limit
budget_status() {
    local value="$1" limit="$2"
    local pct=$((value * 100 / limit))
    if (( value > limit )); then
        echo "FAIL"
    elif (( pct >= BUDGET_WARN_PCT )); then
        echo "WARN"
    else
        echo "PASS"
    fi
}

# Hard enforcement: exit non-zero if either footprint exceeds budget.
check_size_budget() {
    local target="$1" build_type="$2"
    local triple
    triple=$(get_target_triple "$target")
    local elf="$BUILD_ROOT/target/$triple/$build_type/kernel"

    [[ -f "$elf" ]] || error "Size check: binary not found: $elf"

    load_size_budget "$target"

    local rom ram fail=false
    rom=$(compute_rom_footprint "$elf")
    ram=$(compute_ram_footprint "$elf")

    if (( rom > BUDGET_ROM_BYTES )); then
        error "ROM budget exceeded for $target: ${rom}B > ${BUDGET_ROM_BYTES}B"
        fail=true
    fi
    if (( ram > BUDGET_RAM_BYTES )); then
        error "RAM budget exceeded for $target: ${ram}B > ${BUDGET_RAM_BYTES}B"
        fail=true
    fi

    if [[ "$fail" == true ]]; then
        error "Memory-footprint gate FAILED for $target (${BUILD_TYPE})."
        return 1
    fi

    log_success "Size budget OK for $target: ROM ${rom}/${BUDGET_ROM_BYTES}B, RAM ${ram}/${BUDGET_RAM_BYTES}B"
    return 0
}

# Convenience: run the gate for every requested target in one shot (CI use).
check_all_size_budgets() {
    local build_type="${1:-release}"
    local rc=0
    for t in arm arm-v8m riscv riscv-imc riscv64; do
        local triple
        triple=$(get_target_triple "$t" 2>/dev/null) || continue
        local elf="$BUILD_ROOT/target/$triple/$build_type/kernel"
        if [[ -f "$elf" ]]; then
            report_footprint "$t" "$build_type"
            check_size_budget "$t" "$build_type" || rc=1
        else
            log_info "Skipping size gate for $t (no $build_type artifact)"
        fi
    done
    return $rc
}
