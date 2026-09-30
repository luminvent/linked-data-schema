//! `utoipa` backend: describes a linked data type as the JSON-LD document obtained by compacting it
//! against a context made of its own prefix mappings, so `ex:name` stays `ex:name` instead of
//! being expanded to `http://example.com/name`.

use linked_data_core::{
  PredicatePath, RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant, TokenGenerator,
};
use proc_macro_error3::abort;
use proc_macro2::{Literal, TokenStream};
use syn::spanned::Spanned;

use crate::prefixes::{compact, sorted_prefixes};

#[derive(Debug)]
pub(crate) struct ToSchema;

fn json_ld_context(prefixes: &[(String, String)]) -> TokenStream {
  let insert_all_prefixes = prefixes
    .iter()
    .map(|(prefix, namespace)| {
      let prefix = Literal::string(prefix);
      let namespace = Literal::string(namespace);

      quote::quote! {
        let _ = context.insert(#prefix.to_string(), serde_json::Value::String(#namespace.to_string()));
      }
    })
    .collect::<TokenStream>();

  quote::quote! {
    {
      let mut context = serde_json::Map::new();
      #insert_all_prefixes
      serde_json::Value::Object(context)
    }
  }
}

/// Implements `LinkedDataToSchemaField` for a named type, which is referenced with `$ref` and
/// registered once in the components, even when types reference each other.
fn to_schema_field_impl(ident: &syn::Ident) -> TokenStream {
  quote::quote! {
    impl ::linked_data_schema::LinkedDataToSchemaField for #ident {
      fn value_schema() -> ::linked_data_schema::reexports::utoipa::openapi::RefOr<::linked_data_schema::reexports::utoipa::openapi::schema::Schema> {
        use ::linked_data_schema::reexports::utoipa::{ToSchema, openapi::Ref};

        Ref::from_schema_name(<Self as ToSchema>::name()).into()
      }

      fn schemas(
        schemas: &mut Vec<(String, ::linked_data_schema::reexports::utoipa::openapi::RefOr<::linked_data_schema::reexports::utoipa::openapi::schema::Schema>)>,
      ) {
        use ::linked_data_schema::reexports::utoipa::{PartialSchema, ToSchema};

        let name = <Self as ToSchema>::name().to_string();
        if schemas.iter().any(|(registered, _)| *registered == name) {
          return;
        }
        schemas.push((name, <Self as PartialSchema>::schema()));
        <Self as ToSchema>::schemas(schemas);
      }
    }
  }
}

fn to_schema_impl(
  ident: &syn::Ident,
  schema: TokenStream,
  field_types: &[&syn::Type],
) -> TokenStream {
  let name = Literal::string(&ident.to_string());

  quote::quote! {
    impl ::linked_data_schema::reexports::utoipa::PartialSchema for #ident {
      fn schema() -> ::linked_data_schema::reexports::utoipa::openapi::RefOr<::linked_data_schema::reexports::utoipa::openapi::schema::Schema> {
        #[allow(unused_imports)]
        use ::linked_data_schema::{
          reexports::{
            serde_json,
            utoipa::openapi::{
              extensions::ExtensionsBuilder,
              schema::{ObjectBuilder, OneOfBuilder, SchemaType, Type},
            },
          },
          LinkedDataToSchemaField,
        };

        #schema
      }
    }

    impl ::linked_data_schema::reexports::utoipa::ToSchema for #ident {
      fn name() -> ::std::borrow::Cow<'static, str> {
        ::std::borrow::Cow::Borrowed(#name)
      }

      fn schemas(
        schemas: &mut Vec<(String, ::linked_data_schema::reexports::utoipa::openapi::RefOr<::linked_data_schema::reexports::utoipa::openapi::schema::Schema>)>,
      ) {
        #(<#field_types as ::linked_data_schema::LinkedDataToSchemaField>::schemas(schemas);)*
      }
    }
  }
}

/// Adds a property to the `object` builder in scope, required unless its type is optional.
fn property(key: &str, ty: &syn::Type) -> TokenStream {
  let key = Literal::string(key);

  quote::quote! {
    object = object.property(#key, <#ty as LinkedDataToSchemaField>::value_schema());
    if <#ty as LinkedDataToSchemaField>::REQUIRED {
      object = object.required(#key);
    }
  }
}

impl TokenGenerator for ToSchema {
  fn generate_type_tokens(linked_data_type: &RdfType<Self>, tokens: &mut TokenStream) {
    let implementations = match linked_data_type {
      RdfType::Enum(rdf_enum) => quote::quote! {#rdf_enum},
      RdfType::Struct(rdf_struct) => quote::quote! {#rdf_struct},
    };

    tokens.extend(implementations)
  }

  fn generate_struct_tokens(rdf_struct: &RdfStruct<Self>, tokens: &mut TokenStream) {
    let ident = &rdf_struct.ident;

    let Some(type_iri) = rdf_struct.type_iri() else {
      abort!(
        ident.span(),
        "missing type iri, add `#[ld(type = \"...\")]`"
      )
    };

    let prefixes = sorted_prefixes(rdf_struct.prefix_mappings().clone());

    let compacted_type = Literal::string(&compact(type_iri.as_str(), &prefixes));
    let type_iri = Literal::string(type_iri.as_str());
    let context = json_ld_context(&prefixes);

    let mut has_id_field = false;
    let mut properties = TokenStream::new();
    let mut predicates = TokenStream::new();

    let fields = rdf_struct
      .fields
      .iter()
      .filter(|field| !field.is_ignored())
      .collect::<Vec<_>>();

    for field in &fields {
      if field.is_flattened() {
        abort!(
          field.ty.span(),
          "flattened fields are not supported by `LinkedDataToSchema`"
        )
      }

      if field.is_id() {
        has_id_field = true;
        properties.extend(property("@id", &field.ty));
      } else if let Some(predicate) = field.predicate() {
        let key = compact(predicate.as_str(), &prefixes);
        properties.extend(property(&key, &field.ty));

        let key = Literal::string(&key);
        let predicate = Literal::string(predicate.as_str());
        predicates.extend(quote::quote! {
          let _ = predicates.insert(#key.to_string(), serde_json::Value::String(#predicate.to_string()));
        });
      }
    }

    let default_id = (!has_id_field).then(|| {
      quote::quote! {
        object = object.property("@id", ObjectBuilder::new().schema_type(Type::String));
      }
    });

    let schema = quote::quote! {
      let mut predicates = serde_json::Map::new();
      #predicates

      let mut object = ObjectBuilder::new()
        .schema_type(Type::Object)
        .property(
          "@context",
          ObjectBuilder::new()
            .schema_type(SchemaType::AnyValue)
            .examples([#context]),
        )
        .property(
          "@type",
          ObjectBuilder::new()
            .schema_type(Type::String)
            .enum_values(Some([#compacted_type])),
        )
        .required("@type")
        .extensions(Some(
          ExtensionsBuilder::new()
            .add("x-jsonld-type", #type_iri)
            .add("x-jsonld-predicates", serde_json::Value::Object(predicates))
            .build(),
        ));

      #default_id
      #properties

      object.into()
    };

    let field_types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();

    tokens.extend(to_schema_impl(ident, schema, &field_types));
    tokens.extend(to_schema_field_impl(ident));
  }

  fn generate_enum_tokens(rdf_enum: &RdfEnum<Self>, tokens: &mut TokenStream) {
    let ident = &rdf_enum.ident;

    let prefixes = sorted_prefixes(rdf_enum.prefix_mappings().clone());

    let schema = if rdf_enum.is_closed_list() {
      // Each unit variant is a resource, serialized as a node reference: `{"@id": "ex:Variant"}`.
      let resources = rdf_enum
        .variants
        .iter()
        .map(|variant| match variant.predicate_path() {
          PredicatePath::Predicate(iri) => Literal::string(&compact(iri.as_str(), &prefixes)),
          PredicatePath::ChainedPath { .. } => abort!(
            ident.span(),
            "unit variants must be identified by a single IRI"
          ),
        });

      quote::quote! {
        ObjectBuilder::new()
          .schema_type(Type::Object)
          .property(
            "@id",
            ObjectBuilder::new()
              .schema_type(Type::String)
              .enum_values(Some([#(#resources),*])),
          )
          .required("@id")
          .into()
      }
    } else {
      // Each variant wraps its value under its predicate (or chain of predicates).
      let variants = rdf_enum.variants.iter().map(|variant| {
        let ty = variant
          .ty
          .as_ref()
          .expect("unit variants are handled as closed list");

        match variant.predicate_path() {
          PredicatePath::Predicate(iri) => {
            let property = property(&compact(iri.as_str(), &prefixes), ty);

            quote::quote! {
              {
                let mut object = ObjectBuilder::new().schema_type(Type::Object);
                #property
                object
              }
            }
          }
          PredicatePath::ChainedPath {
            to_blank,
            from_blank,
          } => {
            let to_blank = Literal::string(&compact(to_blank.as_str(), &prefixes));
            let property = property(&compact(from_blank.as_str(), &prefixes), ty);

            quote::quote! {
              {
                let mut object = ObjectBuilder::new().schema_type(Type::Object);
                #property
                ObjectBuilder::new()
                  .schema_type(Type::Object)
                  .property(#to_blank, object)
                  .required(#to_blank)
              }
            }
          }
        }
      });

      quote::quote! {
        OneOfBuilder::new()
          #(.item(#variants))*
          .into()
      }
    };

    let variant_types = rdf_enum
      .variants
      .iter()
      .filter_map(|variant| variant.ty.as_ref())
      .collect::<Vec<_>>();

    tokens.extend(to_schema_impl(ident, schema, &variant_types));
    tokens.extend(to_schema_field_impl(ident));
  }

  // Variants and fields are generated by their enum and struct, which hold the prefix mappings
  // needed to compact their predicates.
  fn generate_variant_tokens(_variant: &RdfVariant<Self>, _tokens: &mut TokenStream) {}

  fn generate_field_tokens(_field: &RdfField<Self>, _tokens: &mut TokenStream) {}
}
