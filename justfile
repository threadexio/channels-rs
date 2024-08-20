profile := "perf"
package := "perf"

# Produce a flamegraph of any binary of the `perf` package.
flamegraph BIN OUTPUT="flamegraph.svg":
    echo 1000 | sudo tee /proc/sys/kernel/perf_event_max_sample_rate >/dev/null
    cargo flamegraph --deterministic -F 1000 --profile {{profile}} -o {{OUTPUT}} --package {{package}} --bin {{BIN}}
