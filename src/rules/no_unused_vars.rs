use std::collections::HashSet;
use rslint_parser::{SyntaxKind, SyntaxToken};
use crate::{Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct NoUnusedVars;

impl Rule for NoUnusedVars {
    fn name(&self) -> &'static str {
        "no-unused-vars"
    }

    fn check(&self, rule_context: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        
        let mut declared: HashSet<SyntaxToken> = HashSet::new();
        let mut used: HashSet<SyntaxToken> = HashSet::new();

        for element in rule_context.root.descendants_with_tokens() {
            let token = match element.as_token() {
                Some(t) if t.kind() == SyntaxKind::IDENT => t,
                _ => continue,
            };

            let name = token.text().to_string();
            
            if name == "let" || name == "var" || name == "const" {
                continue;
            }

            let is_direct_name_parent =
                token.parent().kind() == SyntaxKind::NAME;

            let mut has_decl_ancestor = false;

            for ancestor in token.parent().ancestors() {
                if ancestor.kind() == SyntaxKind::VAR_DECL
                    || ancestor.kind() == SyntaxKind::PARAMETER_LIST
                {
                    has_decl_ancestor = true;
                    break;
                }
            }

            if is_direct_name_parent && has_decl_ancestor {
                declared.insert(token.clone());
                continue;
            }
            

            let mut saw_name_ref = false;
            let mut is_used = false;

            for ancestor in token.parent().ancestors() {
                match ancestor.kind() {
                    SyntaxKind::NAME_REF => {
                        saw_name_ref = true;
                    }

                    SyntaxKind::ARG_LIST
                    | SyntaxKind::VAR_DECL
                    | SyntaxKind::RETURN_STMT
                    if saw_name_ref =>
                        {
                            is_used = true;
                            break;
                        }
                    
                    SyntaxKind::CALL_EXPR if saw_name_ref => {
                        break;
                    }

                    _ => {}
                }
            }

            if is_used {
                used.insert(token.clone());
            }
        }
        

        for tkn in declared {
            if !used.contains(&tkn) {
                diagnostics.push(rule_context.diagnostic_at(tkn.text_range(), format!("Unused variable {}", tkn.text())));
            }
        }

        diagnostics
    }
}
