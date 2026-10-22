//! Trigger script sources of a `.scn` file in document order.

use serde::{Deserialize, Serialize};

/// `<ExternalTriggerScript FileName="...">` element.
///
/// `BScenario_load` @ `0x140377be0` (step `X`) gives a non-empty `FileName`
/// to `BTriggerManager_createScriptFromFile` @ `0x140487520` and activates
/// the script at once. That function loads the file from the trigger script
/// directory (`dword_1414B2F98`) and requires a `TriggerSystem` root.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExternalTriggerScript {
    /// Trigger script file name from the `FileName` attribute.
    #[serde(rename = "@FileName", default)]
    pub file_name: String,
}

/// One trigger script that a scenario creates, in document order.
#[derive(Debug, Clone)]
pub enum ScenarioTriggerSource<'a> {
    /// A `<TriggerSystem>` element of the `.scn` file (step `W`).
    Embedded(&'a bdt::Node),
    /// The non-empty `FileName` of an `<ExternalTriggerScript>` element (step `X`).
    External(String),
}

/// List the trigger scripts of a `<Scenario>` root in the engine order.
///
/// `BScenario_load` walks the root children in document order. It creates
/// and activates each `TriggerSystem` and each `ExternalTriggerScript` when
/// it reaches the element. The element and attribute names compare
/// case-insensitively. An `ExternalTriggerScript` without a `FileName`
/// creates no script. The engine skips both steps when it loads a saved game.
#[must_use]
pub fn scenario_trigger_sources(root: &bdt::Node) -> Vec<ScenarioTriggerSource<'_>> {
    root.children
        .iter()
        .filter_map(|child| {
            if child.name.eq_ignore_ascii_case("TriggerSystem") {
                Some(ScenarioTriggerSource::Embedded(child))
            } else if child.name.eq_ignore_ascii_case("ExternalTriggerScript") {
                child
                    .attributes
                    .iter()
                    .find(|attribute| attribute.name.eq_ignore_ascii_case("FileName"))
                    .map(|attribute| attribute.value.to_string_value())
                    .filter(|file_name| !file_name.is_empty())
                    .map(ScenarioTriggerSource::External)
            } else {
                None
            }
        })
        .collect()
}
