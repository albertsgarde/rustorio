//! Build-script helpers for generated resource documentation.
//!
//! Use this module from a crate's `build.rs` to generate the documentation snippets consumed by
//! [`resource_type!`](crate::resource_type) wrappers.
//!
//! ## AI Disclaimer
//! This file is mostly AI-generated and not checked too thoroughly, so expect lower quality than the rest of the codebase.
//! We're accepting this because it's pretty isolated from the rest of the codebase.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    path::PathBuf,
};

use quote::ToTokens;
use syn::{
    self, Attribute, Ident, Item, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Generate resource documentation snippets by scanning all `.rs` files under `src/`.
///
/// Writes snippets to `$OUT_DIR/resource_docs/<ResourceName>.md`.
pub fn generate() -> io::Result<()> {
    let src_dir = PathBuf::from("src");
    let paths = rs_files(&src_dir)?;

    let resource_names = resource_names(&paths)?;
    let recipe_uses = recipe_uses(&paths)?;

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let resource_docs_dir = out_dir.join("resource_docs");
    fs::create_dir_all(&resource_docs_dir)?;

    for resource_name in resource_names {
        let uses = recipe_uses.get(&resource_name);
        fs::write(
            resource_docs_dir.join(format!("{resource_name}.md")),
            resource_doc(uses),
        )?;
    }

    Ok(())
}

fn rs_files(dir: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            paths.extend(rs_files(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            paths.push(path);
        }
    }
    Ok(paths)
}

#[derive(Default)]
struct RecipeUses {
    produced_by: Vec<RecipeUse>,
    used_by: Vec<RecipeUse>,
}

struct RecipeUse {
    recipe: String,
}

struct RecipeItem {
    ty: Type,
}

impl Parse for RecipeItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let _ = content.parse::<Token![,]>()?;
        let ty = content.parse()?;
        Ok(Self { ty })
    }
}

struct RecipeItems(Punctuated<RecipeItem, Token![,]>);

impl Parse for RecipeItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(RecipeItem::parse, Token![,])?))
    }
}

struct ResourceTypeMacro {
    _attrs: Vec<Attribute>,
    ident: Ident,
}

impl Parse for ResourceTypeMacro {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident = input.parse()?;
        Ok(Self {
            _attrs: attrs,
            ident,
        })
    }
}

fn resource_names(paths: &[PathBuf]) -> io::Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();

    for path in paths {
        let source = fs::read_to_string(path)?;
        let file = syn::parse_file(&source).expect("resource modules should parse");

        for item in file.items {
            let Item::Macro(item_macro) = item else {
                continue;
            };

            let Some(macro_name) = item_macro.mac.path.segments.last() else {
                continue;
            };
            if macro_name.ident != "resource_type" && macro_name.ident != "documented_resource_type"
            {
                continue;
            }

            let resource_type = syn::parse2::<ResourceTypeMacro>(item_macro.mac.tokens)
                .expect("resource type macro invocation should contain docs and an identifier");
            names.insert(resource_type.ident.to_string());
        }
    }

    Ok(names)
}

fn recipe_uses(paths: &[PathBuf]) -> io::Result<BTreeMap<String, RecipeUses>> {
    let mut uses = BTreeMap::<String, RecipeUses>::new();

    for path in paths {
        let source = fs::read_to_string(path)?;
        let file = syn::parse_file(&source).expect("recipes module should parse");

        for item in file.items {
            let Item::Struct(item_struct) = item else {
                continue;
            };

            let recipe_name = item_struct.ident.to_string();

            for attr in &item_struct.attrs {
                let direction = if attr.path().is_ident("recipe_inputs") {
                    RecipeDirection::Input
                } else if attr.path().is_ident("recipe_outputs") {
                    RecipeDirection::Output
                } else {
                    continue;
                };

                for item in recipe_items(attr) {
                    let resource_name = resource_name(&item.ty);
                    let recipe_use = RecipeUse {
                        recipe: recipe_name.clone(),
                    };
                    let uses = uses.entry(resource_name).or_default();

                    match direction {
                        RecipeDirection::Input => uses.used_by.push(recipe_use),
                        RecipeDirection::Output => uses.produced_by.push(recipe_use),
                    }
                }
            }
        }
    }

    Ok(uses)
}

enum RecipeDirection {
    Input,
    Output,
}

fn recipe_items(attr: &Attribute) -> Vec<RecipeItem> {
    attr.parse_args::<RecipeItems>()
        .expect("recipe item attributes should parse")
        .0
        .into_iter()
        .collect()
}

fn resource_name(ty: &Type) -> String {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident.to_string();
    }

    ty.to_token_stream().to_string()
}

fn resource_doc(uses: Option<&RecipeUses>) -> String {
    let mut doc = String::new();

    doc.push_str("\n### Produced By\n\n");
    match uses.map(|uses| uses.produced_by.as_slice()) {
        Some([first, rest @ ..]) => {
            push_recipe_use(&mut doc, first);
            for recipe_use in rest {
                push_recipe_use(&mut doc, recipe_use);
            }
        }
        _ => doc.push_str("No recipe produces this resource.\n"),
    }

    doc.push_str("\n### Used By\n\n");
    match uses.map(|uses| uses.used_by.as_slice()) {
        Some([first, rest @ ..]) => {
            push_recipe_use(&mut doc, first);
            for recipe_use in rest {
                push_recipe_use(&mut doc, recipe_use);
            }
        }
        _ => doc.push_str("No recipe uses this resource.\n"),
    }

    doc
}

fn push_recipe_use(doc: &mut String, recipe_use: &RecipeUse) {
    doc.push_str(&format!(
        "- [`{recipe}`](crate::recipes::{recipe})\n",
        recipe = recipe_use.recipe
    ));
}
