//! `bisync_suffix_macro` provides a procedural macro to conditionally append suffixes to method names
//! in `.await` expressions, enabling dual support for asynchronous and blocking code paths.
//!
//! This macro is particularly useful in libraries that need to provide both async and blocking APIs,
//! allowing the same codebase to be compiled for different execution models based on feature flags.
//!
//! For more details, see the documentation for the [`suffix`] macro.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, ExprAwait, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    visit_mut::{self, VisitMut},
};

/// A procedural macro to conditionally append a suffix to method names in `.await` expressions.
///
/// This macro is designed to facilitate writing code that can be compiled for both asynchronous
/// and blocking contexts. It takes a suffix string and an expression, and transforms the expression
/// by appending the suffix to method names in `.await` calls when the `async` feature is enabled.
/// When the `blocking` feature is enabled (and `async` is not), the original expression is used without modification.
///
/// # Usage
///
/// ```ignore
/// suffix!("_async", self.some_method().await)
/// ```
///
/// In the above example, if the `async` feature is enabled, the expression will be transformed to
/// `self.some_method_async().await`. If the `blocking` feature is enabled (and `async` is not),
/// the original `self.some_method().await` will be used, but since it's in a blocking context,
/// you would typically have a corresponding blocking method defined.
///
/// # Note
///
/// - The macro expects exactly two arguments: a string literal (the suffix) and an expression.
/// - The expression must be a valid Rust expression that may contain `.await` calls.
/// - Feature flags `async` and `blocking` must be defined in the crate using this macro for the conditional compilation to work as intended.
#[proc_macro]
pub fn suffix(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as SuffixMacroInput);
    let suffix_value = parsed_input.suffix_str.value();

    let mut async_expr_transformed = parsed_input.expr.clone();
    let blocking_expr_original = parsed_input.expr;

    let mut suffixer = AwaitMethodSuffixer {
        suffix: &suffix_value,
    };
    suffixer.visit_expr_mut(&mut async_expr_transformed);

    let output = quote! {
        {
            #[cfg(feature = "async")]
            {
                #async_expr_transformed
            }
            #[cfg(all(feature = "blocking", not(feature = "async")))]
            {
                #blocking_expr_original
            }
        }
    };

    output.into()
}

struct SuffixMacroInput {
    suffix_str: LitStr,
    _comma: Token![,],
    expr: Expr,
}

impl Parse for SuffixMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(SuffixMacroInput {
            suffix_str: input.parse()?,
            _comma: input.parse()?,
            expr: input.parse()?,
        })
    }
}

struct AwaitMethodSuffixer<'a> {
    suffix: &'a str,
}

impl<'a> VisitMut for AwaitMethodSuffixer<'a> {
    fn visit_expr_await_mut(&mut self, expr_await: &mut ExprAwait) {
        if let Expr::MethodCall(method_call) = &mut *expr_await.base {
            let original_method_ident = method_call.method.clone();
            let new_method_name_str = format!("{}{}", original_method_ident, self.suffix);
            method_call.method = Ident::new(&new_method_name_str, original_method_ident.span());
        }
        visit_mut::visit_expr_await_mut(self, expr_await);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_mut::visit_expr_mut(self, expr);
    }
}
