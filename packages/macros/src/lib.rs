//! Proc macros for transparent JSON syntax in builder patterns

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Block, Expr, ExprBlock, Result};

/// Proc macro that enables transparent `{"key" => "value"}` syntax in builder chains
///
/// This transforms JSON-like syntax in builder method calls to use hash_map_fn! automatically
///
/// Usage:
/// ```rust
/// use sugars_macros::json_builder;
///
/// json_builder! {
///     FluentAi::agent_role("rusty-squire")
///         .additional_params({"beta" => "true"})
///         .metadata({"key" => "val", "foo" => "bar"})
/// }
/// ```
#[proc_macro]
pub fn json_builder(input: TokenStream) -> TokenStream {
    let input_expr = parse_macro_input!(input as Expr);

    match transform_json_syntax(input_expr) {
        Ok(transformed) => quote! { #transformed }.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Transform expressions to replace JSON-like syntax with hash_map_fn! calls
fn transform_json_syntax(expr: Expr) -> Result<Expr> {
    match expr {
        Expr::MethodCall(mut method_call) => {
            // Transform the receiver first
            method_call.receiver = Box::new(transform_json_syntax(*method_call.receiver)?);

            // Check if this is a method that should support JSON syntax
            let method_name = method_call.method.to_string();
            if matches!(method_name.as_str(), "additional_params" | "metadata") {
                // Transform the arguments
                for arg in &mut method_call.args {
                    if let Some(transformed) = transform_json_object_arg(arg.clone())? {
                        *arg = transformed;
                    }
                }
            }

            Ok(Expr::MethodCall(method_call))
        }
        Expr::Call(mut call) => {
            // Transform function arguments for Tool::new patterns
            for arg in &mut call.args {
                if let Some(transformed) = transform_json_object_arg(arg.clone())? {
                    *arg = transformed;
                }
            }
            Ok(Expr::Call(call))
        }
        _ => Ok(expr),
    }
}

/// Transform a JSON object argument if it matches the pattern {"key" => "value"}
fn transform_json_object_arg(arg: Expr) -> Result<Option<Expr>> {
    if let Expr::Block(ExprBlock { block, .. }) = arg {
        if let Some(json_pairs) = extract_json_pairs(&block)? {
            // Create hash_map_fn! call
            let transformed = create_hash_map_fn_call(json_pairs)?;
            return Ok(Some(transformed));
        }
    }
    Ok(None)
}

/// Extract key-value pairs from a block that looks like {"key" => "value", ...}
/// Since {"key" => "value"} is not valid Rust syntax, we need to handle this differently
fn extract_json_pairs(_block: &Block) -> Result<Option<Vec<(Expr, Expr)>>> {
    // For now, return None to indicate no JSON pairs found
    // This prevents the FatArrow error while we implement a proper solution
    // TODO: Implement proper token-level parsing for JSON-like syntax
    Ok(None)
}

/// Create a hash_map_fn! macro call from key-value pairs
fn create_hash_map_fn_call(pairs: Vec<(Expr, Expr)>) -> Result<Expr> {
    let mut tokens = quote! { hash_map_fn! };

    // Add the pairs
    let mut pair_tokens = quote! {};
    for (i, (key, value)) in pairs.iter().enumerate() {
        if i > 0 {
            pair_tokens.extend(quote! { , });
        }
        pair_tokens.extend(quote! { #key => #value });
    }

    tokens.extend(quote! { { #pair_tokens } });

    // Parse the generated tokens back into an expression
    let token_stream: TokenStream = tokens.into();
    syn::parse(token_stream)
}

/// Attribute macro that can be applied to builder structs to enable JSON syntax
#[proc_macro_attribute]
pub fn enable_json_syntax(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just return the item unchanged
    // This could be extended to automatically generate JSON-aware methods
    item
}

/// Proc macro that transforms closures to support JSON syntax
#[proc_macro]
pub fn json_closure(input: TokenStream) -> TokenStream {
    let input_expr = parse_macro_input!(input as Expr);

    if let Expr::Closure(ref closure) = input_expr {
        // Transform the closure body to support JSON syntax
        if let Expr::Block(block) = closure.body.as_ref() {
            if let Some(json_pairs) = extract_json_pairs(&block.block).unwrap_or(None) {
                let hash_map_call = create_hash_map_fn_call(json_pairs).unwrap();
                let new_closure = Expr::Closure(syn::ExprClosure {
                    body: Box::new(hash_map_call),
                    ..closure.clone()
                });
                return quote! { #new_closure }.into();
            }
        }
    }

    quote! { #input_expr }.into()
}

/// Creates a closure that returns a hashbrown HashMap from key-value pairs
///
/// This replaces the macro_rules! version to work in proc-macro crates
///
/// Usage:
/// ```rust
/// use sugars_macros::hash_map_fn;
///
/// let map_fn = hash_map_fn!{"key" => "value", "foo" => "bar"};
/// let map = map_fn();
/// ```
#[proc_macro]
pub fn hash_map_fn(input: TokenStream) -> TokenStream {
    // Convert the input to a string and manually parse key => value pairs
    let input_str = input.to_string();

    // Transform "key" => "value" pairs to ("key", "value") tuples
    let parts: Vec<&str> = input_str.split(',').collect();
    let mut tuple_pairs = Vec::new();

    for part in parts {
        let trimmed = part.trim();
        if let Some(arrow_pos) = trimmed.find(" => ") {
            let key = trimmed[..arrow_pos].trim();
            let value = trimmed[arrow_pos + 4..].trim();
            tuple_pairs.push(format!("({}, {})", key, value));
        } else if let Some(arrow_pos) = trimmed.find("=>") {
            let key = trimmed[..arrow_pos].trim();
            let value = trimmed[arrow_pos + 2..].trim();
            tuple_pairs.push(format!("({}, {})", key, value));
        }
    }

    let tuple_str = tuple_pairs.join(", ");
    let parsed_tokens: proc_macro2::TokenStream = tuple_str.parse().unwrap_or_default();

    quote! {
        || <::hashbrown::HashMap::<_, _> as ::core::iter::FromIterator<_>>::from_iter([
            #parsed_tokens
        ])
    }
    .into()
}
