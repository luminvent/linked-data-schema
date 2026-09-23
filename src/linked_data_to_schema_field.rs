use std::collections::HashSet;
use utoipa::{
  PartialSchema,
  openapi::{
    RefOr,
    schema::{ArrayBuilder, ObjectBuilder, Schema, Type},
  },
};

/// Describes how a field value appears in a compacted JSON-LD document.
///
/// Implemented for the supported primitive and container types, and by
/// `#[derive(LinkedDataToSchema)]` for linked data structs and enums, which are referenced
/// through `$ref` instead of being inlined.
pub trait LinkedDataToSchemaField {
  /// Whether the property must be present in the JSON-LD object.
  const REQUIRED: bool = true;

  fn value_schema() -> RefOr<Schema>;

  /// Registers the named schemas referenced by [`Self::value_schema`].
  fn schemas(_schemas: &mut Vec<(String, RefOr<Schema>)>) {}
}

macro_rules! to_schema_field_impl {
  ($($for_type:ty),*) => {
    $(
      impl LinkedDataToSchemaField for $for_type {
        fn value_schema() -> RefOr<Schema> {
          <$for_type as PartialSchema>::schema()
        }
      }
    )*
  };
}

to_schema_field_impl!(
  String, bool, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, f32, f64
);

impl LinkedDataToSchemaField for ::uuid::Uuid {
  fn value_schema() -> RefOr<Schema> {
    ObjectBuilder::new()
      .schema_type(Type::String)
      .pattern(Some(
        "^urn:uuid:[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-5][0-9a-fA-F]{3}-[089abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$",
      ))
      .into()
  }
}

impl<S: LinkedDataToSchemaField> LinkedDataToSchemaField for Option<S> {
  const REQUIRED: bool = false;

  fn value_schema() -> RefOr<Schema> {
    S::value_schema()
  }

  fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
    S::schemas(schemas)
  }
}

impl<S: LinkedDataToSchemaField> LinkedDataToSchemaField for Box<S> {
  const REQUIRED: bool = S::REQUIRED;

  fn value_schema() -> RefOr<Schema> {
    S::value_schema()
  }

  fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
    S::schemas(schemas)
  }
}

impl<S: LinkedDataToSchemaField> LinkedDataToSchemaField for Vec<S> {
  fn value_schema() -> RefOr<Schema> {
    ArrayBuilder::new().items(S::value_schema()).into()
  }

  fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
    S::schemas(schemas)
  }
}

impl<S: LinkedDataToSchemaField> LinkedDataToSchemaField for HashSet<S> {
  fn value_schema() -> RefOr<Schema> {
    ArrayBuilder::new()
      .items(S::value_schema())
      .unique_items(true)
      .into()
  }

  fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
    S::schemas(schemas)
  }
}
