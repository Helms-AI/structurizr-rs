//! Workspace API bindings for Lua scripts.
//!
//! This module provides Lua-accessible APIs for modifying Structurizr workspaces.
//! The API is designed to be similar to the Structurizr Java/Groovy API for
//! easier migration of existing scripts.

use mlua::{Lua, Result as LuaResult, Table, Value, UserData, UserDataMethods};
use structurizr_core::{Workspace, Container};
use std::sync::{Arc, Mutex};

/// Wrapper around Workspace for Lua access.
/// Uses Arc<Mutex<>> to allow safe mutable access from Lua.
pub struct WorkspaceHandle {
    inner: Arc<Mutex<Workspace>>,
}

impl WorkspaceHandle {
    pub fn new(workspace: Workspace) -> Self {
        Self {
            inner: Arc::new(Mutex::new(workspace)),
        }
    }

    pub fn into_workspace(self) -> Workspace {
        Arc::try_unwrap(self.inner)
            .expect("Workspace still has references")
            .into_inner()
            .expect("Mutex poisoned")
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl UserData for WorkspaceHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Workspace name getter/setter
        methods.add_method("getName", |_, this, ()| {
            let ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            Ok(ws.name.clone())
        });

        methods.add_method_mut("setName", |_, this, name: String| {
            let mut ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            ws.name = name;
            Ok(())
        });

        // Workspace description getter/setter
        methods.add_method("getDescription", |_, this, ()| {
            let ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            Ok(ws.description.clone())
        });

        methods.add_method_mut("setDescription", |_, this, desc: String| {
            let mut ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            ws.description = Some(desc);
            Ok(())
        });

        // Add a person
        methods.add_method_mut("addPerson", |_, this, (name, description): (String, Option<String>)| {
            let mut ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            let desc = description.unwrap_or_default();
            let id = ws.model_mut().add_person(&name, &desc);
            Ok(id.to_string())
        });

        // Add a software system
        methods.add_method_mut("addSoftwareSystem", |_, this, (name, description): (String, Option<String>)| {
            let mut ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            let desc = description.unwrap_or_default();
            let id = ws.model_mut().add_software_system(&name, &desc);
            Ok(id.to_string())
        });

        // Get all people
        methods.add_method("getPeople", |lua, this, ()| {
            let ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            let people: Vec<_> = ws.model().people.iter().map(|p| {
                let t = lua.create_table().unwrap();
                t.set("id", p.id().to_string()).unwrap();
                t.set("name", p.name().to_string()).unwrap();
                t.set("description", p.properties.description.clone()).unwrap();
                t
            }).collect();
            Ok(people)
        });

        // Get all software systems
        methods.add_method("getSoftwareSystems", |lua, this, ()| {
            let ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            let systems: Vec<_> = ws.model().software_systems.iter().map(|s| {
                let t = lua.create_table().unwrap();
                t.set("id", s.id().to_string()).unwrap();
                t.set("name", s.name().to_string()).unwrap();
                t.set("description", s.properties.description.clone()).unwrap();
                t
            }).collect();
            Ok(systems)
        });

        // Add container to a software system (by system name)
        methods.add_method_mut("addContainerByName", |_, this, (system_name, name, description, technology): (String, String, Option<String>, Option<String>)| {
            let mut ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;

            // Find the software system by name
            let system = ws.model_mut().software_systems.iter_mut()
                .find(|s| s.name() == system_name)
                .ok_or_else(|| mlua::Error::external(format!("Software system not found: {}", system_name)))?;

            let mut container = Container::new(&name);
            if let Some(desc) = description {
                container = container.with_description(desc);
            }
            if let Some(tech) = technology {
                container = container.with_technology(tech);
            }
            let id = system.add_container(container);
            Ok(id.to_string())
        });

        // Add relationship between elements (by name)
        methods.add_method_mut("addRelationshipByName", |_, this, (source_name, dest_name, description, technology): (String, String, Option<String>, Option<String>)| {
            let mut ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;

            // Find source element ID by name
            let source_id = find_element_id_by_name(&ws, &source_name)
                .ok_or_else(|| mlua::Error::external(format!("Source element not found: {}", source_name)))?;

            // Find destination element ID by name
            let dest_id = find_element_id_by_name(&ws, &dest_name)
                .ok_or_else(|| mlua::Error::external(format!("Destination element not found: {}", dest_name)))?;

            let desc = description.unwrap_or_default();
            ws.model_mut().add_relationship(source_id, dest_id, &desc, technology);
            Ok(())
        });

        // Find element by name
        methods.add_method("findElementByName", |lua, this, name: String| {
            let ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;

            // Search in people
            for person in &ws.model().people {
                if person.name() == name {
                    let t = lua.create_table()?;
                    t.set("id", person.id().to_string())?;
                    t.set("name", person.name().to_string())?;
                    t.set("type", "Person")?;
                    return Ok(Value::Table(t));
                }
            }

            // Search in software systems
            for system in &ws.model().software_systems {
                if system.name() == name {
                    let t = lua.create_table()?;
                    t.set("id", system.id().to_string())?;
                    t.set("name", system.name().to_string())?;
                    t.set("type", "SoftwareSystem")?;
                    return Ok(Value::Table(t));
                }

                // Search in containers
                for container in &system.containers {
                    if container.name() == name {
                        let t = lua.create_table()?;
                        t.set("id", container.id().to_string())?;
                        t.set("name", container.name().to_string())?;
                        t.set("type", "Container")?;
                        return Ok(Value::Table(t));
                    }
                }
            }

            Ok(Value::Nil)
        });

        // Set property on workspace
        methods.add_method_mut("setProperty", |_, this, (key, value): (String, String)| {
            let mut ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            ws.set_property(key, value);
            Ok(())
        });

        // Get property from workspace
        methods.add_method("getProperty", |_, this, key: String| {
            let ws = this.inner.lock().map_err(|e| mlua::Error::external(e.to_string()))?;
            Ok(ws.get_property(&key).cloned())
        });
    }
}

/// Helper function to find an element ID by name
fn find_element_id_by_name(ws: &Workspace, name: &str) -> Option<structurizr_core::model::ElementId> {
    // Search in people
    for person in &ws.model().people {
        if person.name() == name {
            return Some(person.id());
        }
    }

    // Search in software systems
    for system in &ws.model().software_systems {
        if system.name() == name {
            return Some(system.id());
        }

        // Search in containers
        for container in &system.containers {
            if container.name() == name {
                return Some(container.id());
            }
        }
    }

    None
}

/// Register the workspace API in a Lua state.
pub fn register_workspace_api(lua: &Lua, workspace_handle: WorkspaceHandle) -> LuaResult<()> {
    let globals = lua.globals();
    globals.set("workspace", workspace_handle)?;
    Ok(())
}

/// Register helper functions in the Lua state.
pub fn register_helpers(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    // print function that collects output
    let print_fn = lua.create_function(|_, args: mlua::Variadic<Value>| {
        let output: Vec<String> = args.iter().map(|v| format!("{:?}", v)).collect();
        println!("[Script] {}", output.join("\t"));
        Ok(())
    })?;
    globals.set("print", print_fn)?;

    // Helper to iterate over tables as arrays
    let ipairs_fn = lua.create_function(|lua, table: Table| {
        let iter = lua.create_function(|_, (t, i): (Table, i64)| {
            let next_i = i + 1;
            match t.get::<Value>(next_i) {
                Ok(Value::Nil) => Ok((Value::Nil, Value::Nil)),
                Ok(v) => Ok((Value::Integer(next_i), v)),
                Err(_) => Ok((Value::Nil, Value::Nil)),
            }
        })?;
        Ok((iter, table, 0i64))
    })?;
    globals.set("ipairs", ipairs_fn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_handle_name() {
        let ws = Workspace::new("Test", "Test workspace");
        let handle = WorkspaceHandle::new(ws);

        let lua = Lua::new();
        register_workspace_api(&lua, handle.clone_handle()).unwrap();

        let name: String = lua.load("return workspace:getName()").eval().unwrap();
        assert_eq!(name, "Test");
    }

    #[test]
    fn test_workspace_handle_add_person() {
        let ws = Workspace::new("Test", "Test workspace");
        let handle = WorkspaceHandle::new(ws);

        {
            let lua = Lua::new();
            register_workspace_api(&lua, handle.clone_handle()).unwrap();
            lua.load(r#"workspace:addPerson("Alice", "A user")"#).exec().unwrap();
            // lua is dropped here, releasing the Arc reference
        }

        let workspace = handle.into_workspace();
        assert_eq!(workspace.model().people.len(), 1);
        assert_eq!(workspace.model().people[0].name(), "Alice");
    }

    #[test]
    fn test_workspace_handle_add_system() {
        let ws = Workspace::new("Test", "Test workspace");
        let handle = WorkspaceHandle::new(ws);

        {
            let lua = Lua::new();
            register_workspace_api(&lua, handle.clone_handle()).unwrap();
            lua.load(r#"workspace:addSoftwareSystem("MySystem", "A system")"#).exec().unwrap();
            // lua is dropped here, releasing the Arc reference
        }

        let workspace = handle.into_workspace();
        assert_eq!(workspace.model().software_systems.len(), 1);
        assert_eq!(workspace.model().software_systems[0].name(), "MySystem");
    }
}
