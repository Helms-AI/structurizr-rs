workspace "dtolnay/serde-yaml" "Strongly typed YAML library for Rust" {

    model {
        dtolnayserdeYaml = softwareSystem "dtolnay/serde-yaml" "YAML data format for Serde" {
            serdeYaml = container "serde_yaml" "YAML data format for Serde" "Rust" {
                deserializer = component "Deserializer" "A structure that deserializes YAML into Rust values.\n  # Examples\n  Deserializing a single document:\n  ```\n use anyhow::Result;\n use serde::Deserialize;\n use serde_yaml::Value;\n  fn main() -> Result<(" "Rust Struct"
                progress = component "Progress" "" "Rust Enum"
                event = component "Event" "" "Rust Enum"
                error = component "Error" "An error that happened serializing or deserializing YAML data." "Rust Struct"
                errorImpl = component "ErrorImpl" "" "Rust Enum"
                pos = component "Pos" "" "Rust Struct"
                location = component "Location" "The input location that an error occured." "Rust Struct"
                mapping = component "mapping" "" "Rust Module"
                value = component "value" "" "Rust Module"
                with = component "with" "" "Rust Module"
                cStr = component "CStr" "" "Rust Struct"
                error = component "Error" "" "Rust Enum"
                emitter = component "Emitter" "" "Rust Struct"
                event = component "Event" "" "Rust Enum"
                scalar = component "Scalar" "" "Rust Struct"
                scalarStyle = component "ScalarStyle" "" "Rust Enum"
                sequence = component "Sequence" "" "Rust Struct"
                mapping = component "Mapping" "" "Rust Struct"
                error = component "Error" "" "Rust Struct"
                mark = component "Mark" "" "Rust Struct"
                emitter = component "emitter" "" "Rust Module"
                error = component "error" "" "Rust Module"
                parser = component "parser" "" "Rust Module"
                tag = component "tag" "" "Rust Module"
                parser = component "Parser" "" "Rust Struct"
                event = component "Event" "" "Rust Enum"
                scalar = component "Scalar" "" "Rust Struct"
                sequenceStart = component "SequenceStart" "" "Rust Struct"
                mappingStart = component "MappingStart" "" "Rust Struct"
                anchor = component "Anchor" "" "Rust Struct"
                scalarStyle = component "ScalarStyle" "" "Rust Enum"
                tag = component "Tag" "" "Rust Struct"
                owned = component "Owned" "" "Rust Struct"
                initPtr = component "InitPtr" "" "Rust Struct"
                loader = component "Loader" "" "Rust Struct"
                document = component "Document" "" "Rust Struct"
                mapping = component "Mapping" "A YAML mapping in which the keys and values are both `serde_yaml::Value`." "Rust Struct"
                index = component "Index" "A type that can be used to index into a `serde_yaml::Mapping`." "Rust Trait"
                iter = component "Iter" "Iterator over `&serde_yaml::Mapping`." "Rust Struct"
                iterMut = component "IterMut" "Iterator over `&mut serde_yaml::Mapping`." "Rust Struct"
                intoIter = component "IntoIter" "Iterator over `serde_yaml::Mapping` by value." "Rust Struct"
                keys = component "Keys" "Iterator of the keys of a `&serde_yaml::Mapping`." "Rust Struct"
                intoKeys = component "IntoKeys" "Iterator of the keys of a `serde_yaml::Mapping`." "Rust Struct"
                values = component "Values" "Iterator of the values of a `&serde_yaml::Mapping`." "Rust Struct"
                valuesMut = component "ValuesMut" "Iterator of the values of a `&mut serde_yaml::Mapping`." "Rust Struct"
                intoValues = component "IntoValues" "Iterator of the values of a `serde_yaml::Mapping`." "Rust Struct"
                entry = component "Entry" "Entry for an existing key-value pair or a vacant location to insert one." "Rust Enum"
                occupiedEntry = component "OccupiedEntry" "A view into an occupied entry in a [`Mapping`]." "Rust Struct"
                vacantEntry = component "VacantEntry" "A view into a vacant entry in a [`Mapping`]." "Rust Struct"
                number = component "Number" "Represents a YAML number, whether integer or floating point." "Rust Struct"
                path = component "Path" "Path to the current value in the input, like `dependencies.serde.typo1`." "Rust Enum"
                serializer = component "Serializer" "A structure for serializing Rust values into YAML.\n  # Example\n  ```\n use anyhow::Result;\n use serde::Serialize;\n use std::collections::BTreeMap;\n  fn main() -> Result<()> {\n let mut buffer = Vec::new" "Rust Struct"
                seqDeserializer = component "SeqDeserializer" "" "Rust Struct"
                mapDeserializer = component "MapDeserializer" "" "Rust Struct"
                seqRefDeserializer = component "SeqRefDeserializer" "" "Rust Struct"
                mapRefDeserializer = component "MapRefDeserializer" "" "Rust Struct"
                index = component "Index" "A type that can be used to index into a `serde_yaml::Value`." "Rust Trait"
                tagged = component "tagged" "" "Rust Module"
                value = component "Value" "Represents any valid YAML value." "Rust Enum"
                serializer = component "Serializer" "Serializer whose output is a `Value`.\n  This is the serializer that backs [`serde_yaml::to_value`][crate::to_value].\n Unlike the main serde_yaml serializer which goes from some serializable\n value of" "Rust Struct"
                serializeArray = component "SerializeArray" "" "Rust Struct"
                serializeTupleVariant = component "SerializeTupleVariant" "" "Rust Struct"
                serializeMap = component "SerializeMap" "" "Rust Enum"
                serializeStruct = component "SerializeStruct" "" "Rust Struct"
                serializeStructVariant = component "SerializeStructVariant" "" "Rust Struct"
                tag = component "Tag" "A representation of YAML's `!Tag` syntax, used for enums.\n  Refer to the example code on [`TaggedValue`] for an example of deserializing\n tagged values." "Rust Struct"
                taggedValue = component "TaggedValue" "A `Tag` + `Value` representing a tagged YAML scalar, sequence, or mapping.\n  ```\n use serde_yaml::value::TaggedValue;\n use std::collections::BTreeMap;\n  let yaml = \"\n scalar: !Thing x\n sequence_flow: !Thing [first]\n sequence_block: !Thing\n - first\n mapping_flow: !Thing {k: v}\n mapping_block: !Thing\n k: v\n \";\n  let data: BTreeMap<String, TaggedValue> = serde_yaml::from_str(yaml).unwrap();\n assert!(data[\"scalar\"].tag == \"Thing\");\n assert!(data[\"sequence_flow\"].tag == \"Thing\");\n assert!(data[\"sequence_block\"].tag == \"Thing\");\n assert!(data[\"mapping_flow\"].tag == \"Thing\");\n assert!(data[\"mapping_block\"].tag == \"Thing\");\n  // The leading '!' in tags are not significant." "Rust Struct"
                tagStringVisitor = component "TagStringVisitor" "" "Rust Struct"
                maybeTag = component "MaybeTag" "" "Rust Enum"
                singletonMap = component "singleton_map" "Serialize/deserialize an enum using a YAML map containing one entry in which\n the key identifies the variant name.\n  # Example\n  ```\n # use serde_derive::{Deserialize, Serialize};\n use serde::{Deseria" "Rust Module"
                singletonMapRecursive = component "singleton_map_recursive" "Apply [`singleton_map`] to *all* enums contained within the data structure.\n  # Example\n  ```\n # use serde_derive::{Deserialize, Serialize};\n use serde::{Deserialize, Serialize};\n  #[derive(Serialize," "Rust Module"
            }
        }

        value -> index "Implements" "Rust trait impl"
    }

    views {
        systemContext dtolnayserdeYaml "SystemContext" {
            include *
            autoLayout
        }
        container dtolnayserdeYaml "Containers" {
            include *
            autoLayout
        }
        component serdeYaml "Components_serde_yaml" {
            include *
            autoLayout
        }

        styles {
            element "Person" {
                shape Person
                background "#08427b"
                color "#ffffff"
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "External" {
                background "#999999"
                color "#ffffff"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Component" {
                background "#85bbf0"
                color "#000000"
            }
            element "Database" {
                shape Cylinder
                background "#438dd5"
                color "#ffffff"
            }
        }
    }
}
