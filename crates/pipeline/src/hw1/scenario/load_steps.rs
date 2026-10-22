//! Root elements of a `.scn` file that create players, objects and trigger scripts, in
//! document order.

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

/// One root element of a `.scn` file that creates world state, in document order.
#[derive(Debug, Clone)]
pub enum ScenarioLoadStep<'a> {
    /// The `<Players>` element (step `O`): `BScenario_setupPlayers` creates the players.
    Players,
    /// The `<Objects>` element (step `P`): `BScenario_loadObject` creates each object.
    Objects,
    /// A `<TriggerSystem>` element (step `W`).
    TriggerSystem(&'a bdt::Node),
    /// The non-empty `FileName` of an `<ExternalTriggerScript>` element (step `X`).
    ExternalTriggerScript(String),
}

/// List the root elements of a `<Scenario>` root that create world state, in the engine order.
///
/// `BScenario_load` walks the root children in document order. It creates the players when
/// it reaches `Players`, the objects when it reaches `Objects`, and creates and activates
/// each `TriggerSystem` and each `ExternalTriggerScript` when it reaches the element. The
/// element and attribute names compare case-insensitively. An `ExternalTriggerScript`
/// without a `FileName` creates no script. The engine skips the trigger steps when it loads
/// a saved game.
#[must_use]
pub fn scenario_load_steps(root: &bdt::Node) -> Vec<ScenarioLoadStep<'_>> {
    root.children
        .iter()
        .filter_map(|child| {
            let name = child.name.as_str();
            if name.eq_ignore_ascii_case("Players") {
                Some(ScenarioLoadStep::Players)
            } else if name.eq_ignore_ascii_case("Objects") {
                Some(ScenarioLoadStep::Objects)
            } else if name.eq_ignore_ascii_case("TriggerSystem") {
                Some(ScenarioLoadStep::TriggerSystem(child))
            } else if name.eq_ignore_ascii_case("ExternalTriggerScript") {
                child
                    .attributes
                    .iter()
                    .find(|attribute| attribute.name.eq_ignore_ascii_case("FileName"))
                    .map(|attribute| attribute.value.to_string_value())
                    .filter(|file_name| !file_name.is_empty())
                    .map(ScenarioLoadStep::ExternalTriggerScript)
            } else {
                None
            }
        })
        .collect()
}
