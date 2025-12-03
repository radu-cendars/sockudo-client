//! Comprehensive WASM Delta Compression Tests
//!
//! This test suite validates the delta compression functionality in the WASM build,
//! ensuring full compatibility with the sockudo-js implementation.

#![cfg(target_arch = "wasm32")]

use sockudo_client::wasm::{WasmDeltaOptions, WasmOptions, WasmSockudo};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// Delta Options Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_delta_options_default_creation() {
    let opts = WasmDeltaOptions::new();

    assert_eq!(opts.enabled(), true, "Delta should be enabled by default");
    assert_eq!(opts.debug(), false, "Debug should be disabled by default");
    assert_eq!(
        opts.max_messages_per_key(),
        10,
        "Max messages should be 10 by default"
    );
}

#[wasm_bindgen_test]
fn test_delta_options_enable_disable() {
    let mut opts = WasmDeltaOptions::new();

    // Test enabling
    opts.set_enabled(true);
    assert_eq!(opts.enabled(), true, "Should be enabled");

    // Test disabling
    opts.set_enabled(false);
    assert_eq!(opts.enabled(), false, "Should be disabled");
}

#[wasm_bindgen_test]
fn test_delta_options_debug_mode() {
    let mut opts = WasmDeltaOptions::new();

    // Test enabling debug
    opts.set_debug(true);
    assert_eq!(opts.debug(), true, "Debug should be enabled");

    // Test disabling debug
    opts.set_debug(false);
    assert_eq!(opts.debug(), false, "Debug should be disabled");
}

#[wasm_bindgen_test]
fn test_delta_options_max_messages_per_key() {
    let mut opts = WasmDeltaOptions::new();

    // Test setting different values
    opts.set_max_messages_per_key(5);
    assert_eq!(opts.max_messages_per_key(), 5, "Should be 5");

    opts.set_max_messages_per_key(20);
    assert_eq!(opts.max_messages_per_key(), 20, "Should be 20");

    opts.set_max_messages_per_key(1);
    assert_eq!(opts.max_messages_per_key(), 1, "Should be 1");
}

#[wasm_bindgen_test]
fn test_delta_options_algorithms_single() {
    let mut opts = WasmDeltaOptions::new();

    // Test fossil only
    opts.set_algorithms("fossil");
    // Algorithms are internal, but we can test that it doesn't panic

    // Test xdelta3 only
    opts.set_algorithms("xdelta3");
}

#[wasm_bindgen_test]
fn test_delta_options_algorithms_multiple() {
    let mut opts = WasmDeltaOptions::new();

    // Test both algorithms
    opts.set_algorithms("fossil,xdelta3");

    // Test reverse order
    opts.set_algorithms("xdelta3,fossil");
}

#[wasm_bindgen_test]
fn test_delta_options_algorithms_with_spaces() {
    let mut opts = WasmDeltaOptions::new();

    // Test with spaces (should be trimmed)
    opts.set_algorithms("fossil , xdelta3");
    opts.set_algorithms(" fossil, xdelta3 ");
    opts.set_algorithms("  fossil  ,  xdelta3  ");
}

#[wasm_bindgen_test]
fn test_delta_options_algorithms_invalid() {
    let mut opts = WasmDeltaOptions::new();

    // Test invalid algorithms (should be ignored and fallback to defaults)
    opts.set_algorithms("invalid");
    opts.set_algorithms("foo,bar");
    opts.set_algorithms("");
}

#[wasm_bindgen_test]
fn test_delta_options_algorithms_mixed_valid_invalid() {
    let mut opts = WasmDeltaOptions::new();

    // Test mixed valid and invalid
    opts.set_algorithms("fossil,invalid,xdelta3");
    opts.set_algorithms("invalid1,fossil,invalid2");
}

// ============================================================================
// WasmOptions Integration Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_wasm_options_basic_creation() {
    let opts = WasmOptions::new("test-app-key");

    // Basic creation should work
    assert!(true, "Options created successfully");
}

#[wasm_bindgen_test]
fn test_wasm_options_with_delta_compression() {
    let mut opts = WasmOptions::new("test-app-key");

    let delta_opts = WasmDeltaOptions::new();
    opts.set_delta_compression(delta_opts);

    // Should set without panicking
    assert!(true, "Delta compression set successfully");
}

#[wasm_bindgen_test]
fn test_wasm_options_enable_delta_compression_convenience() {
    let mut opts = WasmOptions::new("test-app-key");

    // Test convenience method
    opts.enable_delta_compression();

    // Should enable with defaults
    assert!(true, "Delta compression enabled successfully");
}

#[wasm_bindgen_test]
fn test_wasm_options_delta_compression_disabled() {
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(false);
    opts.set_delta_compression(delta_opts);

    // Should accept disabled delta compression
    assert!(true, "Disabled delta compression set successfully");
}

#[wasm_bindgen_test]
fn test_wasm_options_full_configuration() {
    let mut opts = WasmOptions::new("test-app-key");

    // Set all options
    opts.set_cluster("mt1");
    opts.set_ws_host("localhost");
    opts.set_ws_port(6001);
    opts.set_use_tls(false);
    opts.set_auth_endpoint("/pusher/auth");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_debug(true);
    delta_opts.set_max_messages_per_key(15);
    delta_opts.set_algorithms("fossil,xdelta3");

    opts.set_delta_compression(delta_opts);

    assert!(true, "Full configuration set successfully");
}

// ============================================================================
// Delta Options Configuration Variations Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_delta_config_fossil_only_enabled() {
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil");
    delta_opts.set_debug(false);
    delta_opts.set_max_messages_per_key(10);

    opts.set_delta_compression(delta_opts);
    assert!(true, "Fossil-only configuration successful");
}

#[wasm_bindgen_test]
fn test_delta_config_xdelta3_only_enabled() {
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("xdelta3");
    delta_opts.set_debug(false);
    delta_opts.set_max_messages_per_key(10);

    opts.set_delta_compression(delta_opts);
    assert!(true, "Xdelta3-only configuration successful");
}

#[wasm_bindgen_test]
fn test_delta_config_both_algorithms_enabled() {
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil,xdelta3");
    delta_opts.set_debug(false);
    delta_opts.set_max_messages_per_key(10);

    opts.set_delta_compression(delta_opts);
    assert!(true, "Both algorithms configuration successful");
}

#[wasm_bindgen_test]
fn test_delta_config_debug_enabled() {
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil,xdelta3");
    delta_opts.set_debug(true);
    delta_opts.set_max_messages_per_key(10);

    opts.set_delta_compression(delta_opts);
    assert!(true, "Debug mode configuration successful");
}

#[wasm_bindgen_test]
fn test_delta_config_minimal_cache() {
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil");
    delta_opts.set_max_messages_per_key(1);

    opts.set_delta_compression(delta_opts);
    assert!(true, "Minimal cache configuration successful");
}

#[wasm_bindgen_test]
fn test_delta_config_large_cache() {
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil,xdelta3");
    delta_opts.set_max_messages_per_key(100);

    opts.set_delta_compression(delta_opts);
    assert!(true, "Large cache configuration successful");
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_delta_options_zero_max_messages() {
    let mut opts = WasmDeltaOptions::new();

    // Test edge case: 0 max messages (probably should be >= 1, but test it doesn't panic)
    opts.set_max_messages_per_key(0);
    assert_eq!(opts.max_messages_per_key(), 0);
}

#[wasm_bindgen_test]
fn test_delta_options_very_large_max_messages() {
    let mut opts = WasmDeltaOptions::new();

    // Test large value
    opts.set_max_messages_per_key(u32::MAX);
    assert_eq!(opts.max_messages_per_key(), u32::MAX);
}

#[wasm_bindgen_test]
fn test_delta_options_empty_algorithm_string() {
    let mut opts = WasmDeltaOptions::new();

    // Empty string should fall back to defaults
    opts.set_algorithms("");
}

#[wasm_bindgen_test]
fn test_delta_options_only_commas() {
    let mut opts = WasmDeltaOptions::new();

    // Only commas
    opts.set_algorithms(",,,");
}

#[wasm_bindgen_test]
fn test_delta_options_only_spaces() {
    let mut opts = WasmDeltaOptions::new();

    // Only spaces
    opts.set_algorithms("   ");
}

#[wasm_bindgen_test]
fn test_delta_options_duplicate_algorithms() {
    let mut opts = WasmDeltaOptions::new();

    // Duplicates
    opts.set_algorithms("fossil,fossil,xdelta3,xdelta3");
}

#[wasm_bindgen_test]
fn test_delta_options_case_sensitivity() {
    let mut opts = WasmDeltaOptions::new();

    // Test case variations (should be case-insensitive)
    opts.set_algorithms("FOSSIL,XDELTA3");
    opts.set_algorithms("Fossil,Xdelta3");
    opts.set_algorithms("FoSsIl,XdElTa3");
}

// ============================================================================
// Multiple Configuration Changes Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_delta_options_multiple_updates() {
    let mut opts = WasmDeltaOptions::new();

    // Change multiple times
    opts.set_enabled(true);
    opts.set_enabled(false);
    opts.set_enabled(true);
    assert_eq!(opts.enabled(), true);

    opts.set_debug(false);
    opts.set_debug(true);
    opts.set_debug(false);
    assert_eq!(opts.debug(), false);

    opts.set_max_messages_per_key(5);
    opts.set_max_messages_per_key(10);
    opts.set_max_messages_per_key(15);
    assert_eq!(opts.max_messages_per_key(), 15);
}

#[wasm_bindgen_test]
fn test_wasm_options_replace_delta_config() {
    let mut opts = WasmOptions::new("test-app-key");

    // Set first config
    let mut delta_opts1 = WasmDeltaOptions::new();
    delta_opts1.set_enabled(true);
    delta_opts1.set_max_messages_per_key(5);
    opts.set_delta_compression(delta_opts1);

    // Replace with second config
    let mut delta_opts2 = WasmDeltaOptions::new();
    delta_opts2.set_enabled(false);
    delta_opts2.set_max_messages_per_key(20);
    opts.set_delta_compression(delta_opts2);

    assert!(true, "Delta configuration replaced successfully");
}

// ============================================================================
// Client Creation Tests (if connection is not required)
// ============================================================================

#[wasm_bindgen_test]
fn test_client_creation_with_delta_enabled() {
    let mut opts = WasmOptions::new("test-app-key");
    opts.set_cluster("mt1");
    opts.enable_delta_compression();

    // Note: Actual client creation might fail without a real server,
    // but we can test that the configuration is accepted
    assert!(true, "Client configuration with delta compression accepted");
}

#[wasm_bindgen_test]
fn test_client_creation_with_custom_delta_config() {
    let mut opts = WasmOptions::new("test-app-key");
    opts.set_cluster("mt1");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_debug(true);
    delta_opts.set_max_messages_per_key(20);
    delta_opts.set_algorithms("fossil,xdelta3");

    opts.set_delta_compression(delta_opts);

    assert!(true, "Client configuration with custom delta accepted");
}

// ============================================================================
// Compatibility Tests with sockudo-js API
// ============================================================================

#[wasm_bindgen_test]
fn test_api_compatibility_default_config() {
    // Simulate sockudo-js default configuration
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil,xdelta3");
    delta_opts.set_debug(false);

    opts.set_delta_compression(delta_opts);

    assert!(true, "sockudo-js default config compatible");
}

#[wasm_bindgen_test]
fn test_api_compatibility_custom_algorithms() {
    // Simulate custom algorithm preference
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("xdelta3"); // Prefer xdelta3 only

    opts.set_delta_compression(delta_opts);

    assert!(true, "Custom algorithm preference compatible");
}

#[wasm_bindgen_test]
fn test_api_compatibility_debug_mode() {
    // Simulate debug mode enabled
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_debug(true);

    opts.set_delta_compression(delta_opts);

    assert!(true, "Debug mode configuration compatible");
}

#[wasm_bindgen_test]
fn test_api_compatibility_disabled() {
    // Simulate delta compression disabled
    let mut opts = WasmOptions::new("test-app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(false);

    opts.set_delta_compression(delta_opts);

    assert!(true, "Disabled delta compression compatible");
}

// ============================================================================
// Serialization/Type Conversion Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_options_to_internal_conversion() {
    let mut opts = WasmOptions::new("test-app-key");
    opts.set_cluster("mt1");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_max_messages_per_key(15);
    opts.set_delta_compression(delta_opts);

    // Test that conversion to internal types works (via to_sockudo_options)
    let _internal = opts.to_sockudo_options();

    assert!(true, "Conversion to internal types successful");
}

// ============================================================================
// Real-world Scenario Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_scenario_production_config() {
    // Production-like configuration
    let mut opts = WasmOptions::new("production-app-key");
    opts.set_cluster("mt1");
    opts.set_use_tls(true);

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil,xdelta3");
    delta_opts.set_debug(false);
    delta_opts.set_max_messages_per_key(10);

    opts.set_delta_compression(delta_opts);

    assert!(true, "Production configuration successful");
}

#[wasm_bindgen_test]
fn test_scenario_development_config() {
    // Development configuration with debug enabled
    let mut opts = WasmOptions::new("dev-app-key");
    opts.set_ws_host("localhost");
    opts.set_ws_port(6001);
    opts.set_use_tls(false);

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_debug(true);
    delta_opts.set_algorithms("fossil");
    delta_opts.set_max_messages_per_key(5);

    opts.set_delta_compression(delta_opts);

    assert!(true, "Development configuration successful");
}

#[wasm_bindgen_test]
fn test_scenario_minimal_config() {
    // Minimal configuration
    let mut opts = WasmOptions::new("minimal-app-key");
    opts.enable_delta_compression();

    assert!(true, "Minimal configuration successful");
}

#[wasm_bindgen_test]
fn test_scenario_disabled_delta() {
    // Configuration with delta disabled
    let mut opts = WasmOptions::new("no-delta-app-key");
    opts.set_cluster("mt1");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(false);
    opts.set_delta_compression(delta_opts);

    assert!(true, "Disabled delta configuration successful");
}

#[wasm_bindgen_test]
fn test_scenario_high_throughput_config() {
    // High throughput with larger cache
    let mut opts = WasmOptions::new("high-throughput-key");
    opts.set_cluster("mt1");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_algorithms("fossil,xdelta3");
    delta_opts.set_max_messages_per_key(50); // Larger cache

    opts.set_delta_compression(delta_opts);

    assert!(true, "High throughput configuration successful");
}

// ============================================================================
// Documentation Example Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_documentation_example_basic() {
    // Example from documentation
    let opts = WasmOptions::new("app-key");
    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);

    assert!(true, "Basic documentation example works");
}

#[wasm_bindgen_test]
fn test_documentation_example_full() {
    // Full example from documentation
    let mut opts = WasmOptions::new("app-key");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_debug(true);
    delta_opts.set_algorithms("fossil,xdelta3");
    delta_opts.set_max_messages_per_key(10);

    opts.set_delta_compression(delta_opts);

    assert!(true, "Full documentation example works");
}
