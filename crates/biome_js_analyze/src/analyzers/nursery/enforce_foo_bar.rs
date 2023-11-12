use crate::semantic_services::Semantic;
use biome_analyze::{context::RuleContext, declare_rule, Ast, Rule, RuleDiagnostic};
use biome_console::markup;
use biome_js_semantic::{Reference, ReferencesExtensions};
use biome_js_syntax::{
    JsIdentifierBinding, JsVariableDeclaration, JsVariableDeclarator, JsVariableDeclaratorList, T,
};
use biome_rowan::AstNode;

declare_rule! {
    /// Succinct description of the rule.
    ///
    /// Put context and details about the rule.
    /// As a starting point, you can take the description of the corresponding _ESLint_ rule (if any).
    ///
    /// Try to stay consistent with the descriptions of implemented rules.
    ///
    /// Add a link to the corresponding ESLint rule (if any):
    ///
    /// Source: https://eslint.org/docs/latest/rules/rule-name
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// var a = 1;
    /// a = 2;
    /// ```
    ///
    /// ## Valid
    ///
    /// ```js
    /// var a = 1;
    /// ```
    ///
    pub(crate) EnforceFooBar {
        version: "1.3.0",
        name: "enforceFooBar",
        recommended: false,
    }
}

impl Rule for EnforceFooBar {
    type Query = Ast<JsVariableDeclarator>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let parent = node
            .parent::<JsVariableDeclaratorList>()?
            .parent::<JsVariableDeclaration>()?;

        // check if a `const` variable declaration
        if parent.is_const() {
            // Check if value of variable is "bar"
            if node.id().ok()?.text() == "foo" {
                // Check if value of variable is "bar"
                let init_exp = node.initializer()?.expression().ok()?;
                let literal_exp = init_exp.as_any_js_literal_expression()?;
                if literal_exp.as_static_value()?.text() != "bar" {
                    // Report diagnostic to Biome
                    // The details of diagnostic is implemented by "diagnostic" method
                    return Some(());
                }
            }
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Variable is read here."
                },
            )
            .note(markup! {
                "This note will give you more information."
            }),
        )
    }
}
