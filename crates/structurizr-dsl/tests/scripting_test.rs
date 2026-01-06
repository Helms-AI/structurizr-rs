//! Integration tests for script execution.
//!
//! These tests require the 'scripting' feature to be enabled.

#![cfg(feature = "scripting")]

use structurizr_dsl::parse;

#[test]
fn test_script_modifies_workspace_name() {
    let dsl = r##"
workspace "Original Name" {
    !script lua {
        workspace:setName("Modified by Lua Script")
    }
    model {
        user = person "User"
    }
}
"##;

    let workspace = parse(dsl).unwrap();

    // With scripting enabled, the script should have modified the name
    assert_eq!(workspace.name, "Modified by Lua Script");
    // Model should still be intact
    assert_eq!(workspace.model().people.len(), 1);
}

#[test]
fn test_script_adds_elements() {
    let dsl = r##"
workspace "Scripted Workspace" {
    !script lua {
        workspace:addPerson("Scripted User", "Created by script")
        workspace:addSoftwareSystem("Scripted System", "Also created by script")
    }
    model {
        existing = person "Existing User"
    }
}
"##;

    let workspace = parse(dsl).unwrap();

    // Should have 2 people: one from DSL, one from script
    assert_eq!(workspace.model().people.len(), 2);
    // Should have 1 software system from script
    assert_eq!(workspace.model().software_systems.len(), 1);

    // Verify the scripted elements exist
    assert!(workspace.model().people.iter().any(|p| p.name() == "Scripted User"));
    assert!(workspace.model().software_systems.iter().any(|s| s.name() == "Scripted System"));
}

#[test]
fn test_groovy_script_transpilation() {
    let dsl = r##"
workspace "Groovy Test" {
    !script groovy {
        workspace.name = "Modified by Groovy"
        def message = "Hello from Groovy"
        println(message)
    }
    model {
    }
}
"##;

    // Groovy scripts are transpiled to Lua and executed
    let workspace = parse(dsl).unwrap();
    assert_eq!(workspace.name, "Modified by Groovy");
}

#[test]
fn test_multiple_scripts() {
    let dsl = r##"
workspace "Multi-Script" {
    !script lua {
        workspace:setName("First Script")
    }
    !script lua {
        workspace:setDescription("Second Script")
    }
    model {
    }
}
"##;

    let workspace = parse(dsl).unwrap();
    assert_eq!(workspace.name, "First Script");
    assert_eq!(workspace.description, Some("Second Script".to_string()));
}
