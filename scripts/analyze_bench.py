import os
import json
import glob

def get_mean(path):
    with open(path, 'r') as f:
        data = json.load(f)
        return data['mean']['point_estimate'] # nanoseconds

def main():
    results = {}
    # Criterion saves results in target/criterion/{name}/new/estimates.json
    paths = glob.glob("target/criterion/*/new/estimates.json")

    if not paths:
        print("No benchmark results found. Run 'cargo bench' first.")
        return

    for p in paths:
        # Path format: target/criterion/Traversal/Scalar 1M/new/estimates.json
        parts = p.split(os.sep)
        if len(parts) >= 4:
            name = parts[-3]
            results[name] = get_mean(p)

    # Process matches
    output = "### 🚀 Performance Report (SIMD vs Scalar)\n\n"
    output += "| Workload | Scalar Time | SIMD Time | **Speedup** |\n"
    output += "| :--- | :--- | :--- | :--- |\n"

    # Find unique workloads from names like "Scalar 1M", "SIMD 1M"
    workloads = set()
    for name in results.keys():
        name_parts = name.split(' ')
        if len(name_parts) >= 2:
            workloads.add(" ".join(name_parts[1:]))

    sorted_workloads = sorted(list(workloads))

    found_any = False
    for w in sorted_workloads:
        scalar = results.get(f"Scalar {w}")
        simd = results.get(f"SIMD {w}")

        if scalar is not None and simd is not None:
            found_any = True
            ratio = scalar / simd
            s_time = f"{scalar/1000:,.2f} µs" if scalar > 1000 else f"{scalar:.2f} ns"
            i_time = f"{simd/1000:,.2f} µs" if simd > 1000 else f"{simd:.2f} ns"
            emoji = "✅" if ratio > 10 else "⚠️"
            output += f"| Traversal ({w}) | {s_time} | {i_time} | {emoji} **{ratio:.2f}x** |\n"

    if found_any:
        print(output)
    else:
        print("No matching Scalar/SIMD pairs found in results.")

if __name__ == "__main__":
    main()
