import SwiftUI
import UIKit

struct BenchmarkResult: Identifiable, Codable {
    let id: UUID
    let timestamp: Date
    let name: String
    let value: String
    let detail: String

    init(name: String, value: String, detail: String) {
        self.id = UUID()
        self.timestamp = Date()
        self.name = name
        self.value = value
        self.detail = detail
    }
}

struct ContentView: View {
    @State private var results: [BenchmarkResult] = []
    @State private var isRunning: Bool = false
    @State private var currentTest: String = ""
    @State private var sustainedDuration: Double = 5.0
    @State private var showingShareSheet = false
    @State private var exportText = ""

    var body: some View {
        NavigationView {
            List {
                Section("Quick Tests") {
                    Button(action: { runTest("single") }) {
                        TestRow(title: "Single Hash", subtitle: "Cold start performance", icon: "1.circle")
                    }
                    .disabled(isRunning)

                    Button(action: { runTest("warmup") }) {
                        TestRow(title: "Warmed Up (10 hashes)", subtitle: "After warmup, measure 10", icon: "flame")
                    }
                    .disabled(isRunning)

                    Button(action: { runTest("burst100") }) {
                        TestRow(title: "Burst 100 Hashes", subtitle: "Standard benchmark", icon: "bolt")
                    }
                    .disabled(isRunning)
                }

                Section("Sustained Tests") {
                    HStack {
                        Text("Duration")
                        Spacer()
                        Picker("", selection: $sustainedDuration) {
                            Text("5s").tag(5.0)
                            Text("10s").tag(10.0)
                            Text("30s").tag(30.0)
                            Text("60s").tag(60.0)
                        }
                        .pickerStyle(.segmented)
                        .frame(width: 200)
                    }

                    Button(action: { runTest("sustained") }) {
                        TestRow(title: "Sustained Mining", subtitle: "Continuous hashing for \(Int(sustainedDuration))s", icon: "timer")
                    }
                    .disabled(isRunning)
                }

                Section("Stress Tests") {
                    Button(action: { runTest("thermal") }) {
                        TestRow(title: "Thermal Test (60s)", subtitle: "Check for throttling", icon: "thermometer.sun")
                    }
                    .disabled(isRunning)

                    Button(action: { runTest("memory") }) {
                        TestRow(title: "Memory Pressure", subtitle: "Allocate multiple hashers", icon: "memorychip")
                    }
                    .disabled(isRunning)
                }

                if isRunning {
                    Section("Running") {
                        HStack {
                            ProgressView()
                            Text(currentTest)
                                .foregroundColor(.secondary)
                                .padding(.leading, 8)
                        }
                    }
                }

                if !results.isEmpty {
                    Section("Results") {
                        ForEach(results) { result in
                            VStack(alignment: .leading, spacing: 4) {
                                HStack {
                                    Text(result.name)
                                        .font(.headline)
                                    Spacer()
                                    Text(result.value)
                                        .font(.system(.title2, design: .rounded))
                                        .fontWeight(.bold)
                                        .foregroundColor(.blue)
                                }
                                Text(result.detail)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                                Text(result.timestamp, style: .time)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                            .padding(.vertical, 4)
                        }
                    }

                    Section {
                        Button(action: exportLogs) {
                            Label("Export Logs", systemImage: "square.and.arrow.up")
                        }

                        Button("Clear Results", role: .destructive) {
                            results.removeAll()
                        }
                    }
                }
            }
            .navigationTitle("UHash Benchmark")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { runTest("all") }) {
                        Label("Run All", systemImage: "play.fill")
                    }
                    .disabled(isRunning)
                }
            }
            .sheet(isPresented: $showingShareSheet) {
                ShareSheet(text: exportText)
            }
        }
    }

    struct TestRow: View {
        let title: String
        let subtitle: String
        let icon: String

        var body: some View {
            HStack {
                Image(systemName: icon)
                    .foregroundColor(.blue)
                    .frame(width: 30)
                VStack(alignment: .leading) {
                    Text(title)
                    Text(subtitle)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
    }

    func exportLogs() {
        let deviceInfo = getDeviceInfo()
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd HH:mm:ss"

        var log = """
        ═══════════════════════════════════════════
        UHash Benchmark Report
        ═══════════════════════════════════════════

        Device Information
        ──────────────────
        \(deviceInfo)

        Algorithm: UniversalHash v4
        - Chains: 4
        - Scratchpad: 512KB × 4 = 2MB
        - Rounds: 12,288 per chain
        - Primitives: AES + SHA256 + BLAKE3

        Benchmark Results
        ─────────────────

        """

        for result in results.reversed() {
            log += """
            [\(dateFormatter.string(from: result.timestamp))]
            Test: \(result.name)
            Result: \(result.value)
            Details: \(result.detail)

            """
        }

        log += """
        ═══════════════════════════════════════════
        Generated: \(dateFormatter.string(from: Date()))
        ═══════════════════════════════════════════
        """

        exportText = log
        showingShareSheet = true
    }

    func getDeviceInfo() -> String {
        let device = UIDevice.current
        var systemInfo = utsname()
        uname(&systemInfo)
        let modelCode = withUnsafePointer(to: &systemInfo.machine) {
            $0.withMemoryRebound(to: CChar.self, capacity: 1) {
                String(validatingUTF8: $0) ?? "Unknown"
            }
        }

        let processInfo = ProcessInfo.processInfo

        return """
        Model: \(modelCode) (\(device.name))
        OS: \(device.systemName) \(device.systemVersion)
        Cores: \(processInfo.processorCount)
        Memory: \(processInfo.physicalMemory / 1024 / 1024 / 1024) GB
        """
    }

    func runTest(_ test: String) {
        isRunning = true

        DispatchQueue.global(qos: .userInitiated).async {
            switch test {
            case "single":
                runSingleHash()
            case "warmup":
                runWarmupTest()
            case "burst100":
                runBurst(iterations: 100)
            case "sustained":
                runSustained(seconds: sustainedDuration)
            case "thermal":
                runThermalTest()
            case "memory":
                runMemoryTest()
            case "all":
                runAllTests()
            default:
                break
            }

            DispatchQueue.main.async {
                isRunning = false
                currentTest = ""
            }
        }
    }

    func runSingleHash() {
        updateStatus("Single hash (cold start)...")

        let start = CFAbsoluteTimeGetCurrent()
        let _ = uhash_benchmark(1)
        let elapsed = CFAbsoluteTimeGetCurrent() - start

        let ms = elapsed * 1000
        let rate = 1.0 / elapsed

        addResult(
            name: "Single Hash",
            value: String(format: "%.1f ms", ms),
            detail: String(format: "Cold start: %.1f H/s equivalent", rate)
        )
    }

    func runWarmupTest() {
        updateStatus("Warming up...")

        // Warmup
        let _ = uhash_benchmark(5)

        updateStatus("Measuring...")

        // Measure
        let micros = uhash_benchmark(10)
        let rate = uhash_hashrate(10, micros)

        addResult(
            name: "Warmed Up",
            value: String(format: "%.0f H/s", rate),
            detail: String(format: "10 hashes in %.1f ms after warmup", Double(micros) / 1000.0)
        )
    }

    func runBurst(iterations: UInt32) {
        updateStatus("Running \(iterations) hashes...")

        let micros = uhash_benchmark(iterations)
        let rate = uhash_hashrate(iterations, micros)
        let perHash = Double(micros) / Double(iterations) / 1000.0

        addResult(
            name: "Burst \(iterations)",
            value: String(format: "%.0f H/s", rate),
            detail: String(format: "%.2f ms/hash, total %.1f s", perHash, Double(micros) / 1_000_000.0)
        )
    }

    func runSustained(seconds: Double) {
        updateStatus("Sustained test for \(Int(seconds))s...")

        let targetMicros = UInt64(seconds * 1_000_000)
        var totalHashes: UInt32 = 0
        var totalMicros: UInt64 = 0
        var minRate: Double = Double.infinity
        var maxRate: Double = 0

        // Warmup
        let _ = uhash_benchmark(3)

        // Run in chunks of 10 hashes
        while totalMicros < targetMicros {
            let chunkMicros = uhash_benchmark(10)
            let chunkRate = uhash_hashrate(10, chunkMicros)

            totalHashes += 10
            totalMicros += chunkMicros

            minRate = min(minRate, chunkRate)
            maxRate = max(maxRate, chunkRate)

            let progress = Double(totalMicros) / Double(targetMicros) * 100
            updateStatus(String(format: "Sustained: %.0f%% (%.0f H/s)", progress, chunkRate))
        }

        let avgRate = uhash_hashrate(totalHashes, totalMicros)

        addResult(
            name: "Sustained \(Int(seconds))s",
            value: String(format: "%.0f H/s", avgRate),
            detail: String(format: "%d hashes, range: %.0f-%.0f H/s", totalHashes, minRate, maxRate)
        )
    }

    func runThermalTest() {
        updateStatus("Thermal test (60s)...")

        var samples: [(time: Double, rate: Double)] = []
        let startTime = CFAbsoluteTimeGetCurrent()

        // Warmup
        let _ = uhash_benchmark(3)

        // Run for 60 seconds, sampling every 5 seconds
        for i in 0..<12 {
            let chunkMicros = uhash_benchmark(50)
            let chunkRate = uhash_hashrate(50, chunkMicros)
            let elapsed = CFAbsoluteTimeGetCurrent() - startTime

            samples.append((time: elapsed, rate: chunkRate))
            updateStatus(String(format: "Thermal: %ds (%.0f H/s)", (i + 1) * 5, chunkRate))
        }

        let firstRate = samples.first?.rate ?? 0
        let lastRate = samples.last?.rate ?? 0
        let changePercent = ((lastRate - firstRate) / firstRate) * 100

        var detail = "Start: \(String(format: "%.0f", firstRate)) H/s → End: \(String(format: "%.0f", lastRate)) H/s"
        if changePercent < -5 {
            detail += String(format: " (%.0f%% throttling)", abs(changePercent))
        } else if changePercent > 5 {
            detail += String(format: " (+%.0f%% boost)", changePercent)
        } else {
            detail += " (stable)"
        }

        addResult(
            name: "Thermal (60s)",
            value: String(format: "%.0f%%", 100 + changePercent),
            detail: detail
        )
    }

    func runMemoryTest() {
        updateStatus("Memory pressure test...")

        // Create multiple hashers to stress memory (each is ~2MB)
        var hashers: [OpaquePointer?] = []
        var count = 0

        for i in 1...10 {
            updateStatus("Allocating hasher \(i)/10...")
            if let h = uhash_new() {
                hashers.append(h)
                count += 1
            }
        }

        // Run benchmark with all hashers allocated
        updateStatus("Running with \(count * 2)MB allocated...")
        let micros = uhash_benchmark(50)
        let rate = uhash_hashrate(50, micros)

        // Free hashers
        for h in hashers {
            uhash_free(h)
        }

        addResult(
            name: "Memory Pressure",
            value: String(format: "%.0f H/s", rate),
            detail: "\(count) hashers (\(count * 2)MB) allocated during test"
        )
    }

    func runAllTests() {
        runSingleHash()
        Thread.sleep(forTimeInterval: 0.5)

        runWarmupTest()
        Thread.sleep(forTimeInterval: 0.5)

        runBurst(iterations: 100)
        Thread.sleep(forTimeInterval: 0.5)

        runSustained(seconds: 10)
        Thread.sleep(forTimeInterval: 0.5)

        runMemoryTest()
    }

    func updateStatus(_ status: String) {
        DispatchQueue.main.async {
            currentTest = status
        }
    }

    func addResult(name: String, value: String, detail: String) {
        DispatchQueue.main.async {
            results.insert(BenchmarkResult(name: name, value: value, detail: detail), at: 0)
        }
    }
}

struct ShareSheet: UIViewControllerRepresentable {
    let text: String

    func makeUIViewController(context: Context) -> UIActivityViewController {
        let activityItems: [Any] = [text]
        let controller = UIActivityViewController(activityItems: activityItems, applicationActivities: nil)
        return controller
    }

    func updateUIViewController(_ uiViewController: UIActivityViewController, context: Context) {}
}

#Preview {
    ContentView()
}
