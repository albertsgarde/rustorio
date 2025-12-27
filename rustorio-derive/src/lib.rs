use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, DeriveInput, Ident, LitInt, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct RecipeItemAttrArgs(LitInt, Type);

impl Parse for RecipeItemAttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let amount = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let ty = content.parse()?;
        Ok(Self(amount, ty))
    }
}

struct RecipeItemsAttr(Punctuated<RecipeItemAttrArgs, Token![,]>);

impl Parse for RecipeItemsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input.parse_terminated(RecipeItemAttrArgs::parse, Token![,])?,
        ))
    }
}

struct DeriveRecipeOneway {
    per_type: Vec<(u32, Type)>,
    item_type_ident: Ident,
    amount_const_ident: Ident,
}

impl DeriveRecipeOneway {
    fn new(
        attr: &Attribute,
        attr_name: &str,
        item_type_name: &str,
        amount_const_name: &str,
    ) -> Self {
        let Ok(inner) = attr.parse_args::<RecipeItemsAttr>() else {
            panic!("Invalid \"{attr_name}\" args");
        };

        let per_type = inner
            .0
            .iter()
            .map(|RecipeItemAttrArgs(lit, ty)| {
                let amount = lit
                    .base10_parse::<u32>()
                    .unwrap_or_else(|_| panic!("Invalid amount in \"{attr_name}\" args"));
                (amount, ty.to_owned())
            })
            .collect::<Vec<_>>();
        let item_type_ident = Ident::new(item_type_name, Span::call_site());
        let amount_const_ident = Ident::new(amount_const_name, Span::call_site());

        Self {
            per_type,
            item_type_ident,
            amount_const_ident,
        }
    }

    fn new_inputs(attr: &Attribute) -> Self {
        Self::new(attr, "recipe_inputs", "Inputs", "INPUT_AMOUNTS")
    }

    fn new_outputs(attr: &Attribute) -> Self {
        Self::new(attr, "recipe_outputs", "Outputs", "OUTPUT_AMOUNTS")
    }
}

fn derive_recipe_oneway(oneway: &DeriveRecipeOneway, amount_type_name: &str) -> TokenStream {
    let DeriveRecipeOneway {
        per_type,
        item_type_ident,
        amount_const_ident,
    } = oneway;

    let amount_type_ident = Ident::new(amount_type_name, Span::call_site());
    let amount_types = per_type.iter().map(|_| quote! {u32}).collect::<Vec<_>>();
    let amounts = per_type.iter().map(|(amount, _)| amount);

    let recipe_items = per_type
        .iter()
        .map(|(amount, ty)| quote! {::rustorio_engine::recipe::RecipeItem<#amount, #ty>});

    quote! {
        type #item_type_ident = (#(#recipe_items,)*);

        type #amount_type_ident = (#(#amount_types,)*);
        const #amount_const_ident: (#(#amount_types,)*) = (#(#amounts,)*);
    }
}

fn derive_recipe_ex_oneway(
    oneway: &DeriveRecipeOneway,
    new_fn_name: &str,
    iter_fn_name: &str,
) -> TokenStream {
    let DeriveRecipeOneway {
        per_type,
        item_type_ident,
        amount_const_ident,
    } = oneway;

    let new_fn_ident = Ident::new(new_fn_name, Span::call_site());
    let new_values = per_type
        .iter()
        .map(|_| quote! {::rustorio_engine::recipe::RecipeItem::default()});

    let iter_fn_ident = Ident::new(iter_fn_name, Span::call_site());
    let iter_values = per_type
        .iter()
        .enumerate()
        .map(|(i, (_amount, resource_type))| {
            let i = LitInt::new(&i.to_string(), Span::call_site());
            quote! {(
                <#resource_type as ::rustorio_engine::ResourceType>::NAME,
                Self::#amount_const_ident.#i,
                ::rustorio_engine::recipe::recipe_item_amount(&mut items.#i)
            )}
        });

    quote! {
        fn #new_fn_ident() -> Self::#item_type_ident {
            (#(#new_values,)*)
        }

        fn #iter_fn_ident(
            items: &mut Self::#item_type_ident
        ) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
            [#(#iter_values,)*].into_iter()
        }
    }
}
fn derive_recipe_inner(input: DeriveInput) -> TokenStream {
    let mut inputs = None;
    let mut outputs = None;
    let mut ticks = None;
    for attr in &input.attrs {
        if attr.path().is_ident("recipe_inputs") {
            inputs = Some(derive_recipe_oneway(
                &DeriveRecipeOneway::new_inputs(attr),
                "InputAmountsType",
            ));
        } else if attr.path().is_ident("recipe_outputs") {
            outputs = Some(derive_recipe_oneway(
                &DeriveRecipeOneway::new_outputs(attr),
                "OutputAmountsType",
            ));
        } else if attr.path().is_ident("recipe_ticks") {
            ticks = Some(
                attr.parse_args::<LitInt>()
                    .expect("Invalid \"recipe_ticks\" value"),
            );
        }
    }
    let inputs = inputs.expect("Missing \"recipe_inputs\" attribute");
    let outputs = outputs.expect("Missing \"recipe_outputs\" attribute");
    let ticks = ticks.expect("Missing \"recipe_ticks\" attribute");

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::rustorio_engine::recipe::Recipe for #name #ty_generics #where_clause {
            const TIME: u64 = #ticks;

            #inputs
            #outputs
        }
    }
}

fn derive_recipe_ex_inner(input: DeriveInput) -> TokenStream {
    let mut inputs = None;
    let mut outputs = None;
    for attr in &input.attrs {
        if attr.path().is_ident("recipe_inputs") {
            inputs = Some(derive_recipe_ex_oneway(
                &DeriveRecipeOneway::new_inputs(attr),
                "new_inputs",
                "iter_inputs",
            ));
        } else if attr.path().is_ident("recipe_outputs") {
            outputs = Some(derive_recipe_ex_oneway(
                &DeriveRecipeOneway::new_outputs(attr),
                "new_outputs",
                "iter_outputs",
            ));
        }
    }
    let inputs = inputs.expect("Missing \"recipe_inputs\" attribute");
    let outputs = outputs.expect("Missing \"recipe_outputs\" attribute");

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::rustorio_engine::recipe::RecipeEx for #name #ty_generics #where_clause {
            #inputs
            #outputs
        }
    }
}

#[proc_macro_derive(Recipe, attributes(recipe_inputs, recipe_outputs, recipe_ticks))]
pub fn derive_recipe(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = derive_recipe_inner(input);
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(RecipeEx, attributes(recipe_inputs, recipe_outputs, recipe_ticks))]
pub fn derive_recipe_ex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = derive_recipe_ex_inner(input);
    proc_macro::TokenStream::from(output)
}
