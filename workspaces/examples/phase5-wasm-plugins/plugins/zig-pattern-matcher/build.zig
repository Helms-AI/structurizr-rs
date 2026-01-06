// Zig Build Configuration for WASM Plugin
// =========================================
//
// This file configures the Zig build system to produce a WASM binary.
// Run with: zig build

const std = @import("std");

pub fn build(b: *std.Build) void {
    // Target: WebAssembly (freestanding, no OS)
    const target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .freestanding,
    });

    // Optimization: Release small for minimal binary size
    const optimize = b.standardOptimizeOption(.{});

    // Create the WASM library
    const lib = b.addExecutable(.{
        .name = "plugin",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = if (optimize == .Debug) .ReleaseSmall else optimize,
    });

    // WASM-specific settings
    lib.entry = .disabled; // No entry point (we export _start)
    lib.rdynamic = true; // Export all public symbols

    // Install the artifact
    b.installArtifact(lib);

    // Add a run step for testing (won't work for WASM, but useful structure)
    const run_cmd = b.addRunArtifact(lib);
    run_cmd.step.dependOn(b.getInstallStep());

    const run_step = b.step("run", "Run the plugin");
    run_step.dependOn(&run_cmd.step);
}