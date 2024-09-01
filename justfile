perf_freq := env("PERF_SAMPLE_FREQ", "1000")
cargo_profile := env("PROFILE", "perf")
cargo_package := env("PACKAGE", "perf")

# Configure the kernel to allow using perf.
setup-perf:
    echo {{perf_freq}} | sudo tee /proc/sys/kernel/perf_event_max_sample_rate >/dev/null
    echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid

# Build the binary `BIN` from the specified cargo package.
cargo-build-bin BIN:
    cargo build --package {{cargo_package}} --bin {{BIN}} --profile {{cargo_profile}}

# Produce a flamegraph of any binary of the specified cargo package.
flamegraph BIN OUTPUT="flamegraph.svg": setup-perf
    cargo flamegraph --deterministic -F {{perf_freq}} --profile {{cargo_profile}} -o {{OUTPUT}} --package {{cargo_package}} --bin {{BIN}}

# Record a binary run with `perf record`.
perf-record BIN *ARGS: (cargo-build-bin BIN) setup-perf
    perf record -F {{perf_freq}} -g --call-graph=dwarf,16384 {{ARGS}} -- ./target/{{cargo_profile}}/{{BIN}}

# Export `perf.data` to a format readable by Firefox Profiler.
export-ff-profiler:
    perf script report gecko --save-only firefox_profiler.json
    @echo 'Open https://profiler.firefox.com/ and load the file `{{justfile_directory()/"firefox_profiler.json"}}`'

# Export `perf.data` to a simple flamegraph
export-flamegraph:
    perf script report flamegraph
    @echo 'Open the file `{{justfile_directory()/"flamegraph.html"}}` in your browser.'
