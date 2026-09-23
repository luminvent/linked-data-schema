use linked_data_schema::{LinkedDataToSchema, reexports::uuid};
use serde_json::json;
use utoipa::{
  OpenApi, PartialSchema, ToSchema,
  openapi::{RefOr, schema::Schema},
};

#[allow(dead_code)]
#[derive(LinkedDataToSchema)]
#[ld(prefix("ex" = "http://example.com/"))]
#[ld(type = "ex:Struct")]
struct Struct {
  #[ld(id)]
  id: uuid::Uuid,

  #[ld("ex:field_0")]
  field_0: String,

  #[ld("ex:field_1")]
  field_1: Option<u64>,

  #[ld("ex:field_2")]
  field_2: Vec<SubStruct>,

  #[ld("ex:field_3")]
  field_3: Color,

  #[ld("ex:field_4")]
  field_4: Option<Value>,

  #[ld(ignore)]
  ignored: (),
}

#[allow(dead_code)]
#[derive(LinkedDataToSchema)]
#[ld(prefix("ex" = "http://example.com/"))]
#[ld(prefix("sub" = "http://example.com/sub/"))]
#[ld(type = "sub:SubStruct")]
struct SubStruct {
  #[ld("sub:sub_field_0")]
  sub_field_0: String,

  #[ld("http://other.org/sub_field_1")]
  sub_field_1: Option<Box<SubStruct>>,
}

#[allow(dead_code)]
#[derive(LinkedDataToSchema)]
#[ld(prefix("ex" = "http://example.com/"))]
enum Color {
  #[ld("ex:Red")]
  Red,
  #[ld("ex:Blue")]
  Blue,
}

#[allow(dead_code)]
#[derive(LinkedDataToSchema)]
#[ld(prefix("ex" = "http://example.com/"))]
enum Value {
  #[ld("ex:text")]
  Text(String),
  #[ld("ex:number")]
  Number(#[ld("ex:value")] i64),
}

fn to_json(schema: RefOr<Schema>) -> serde_json::Value {
  serde_json::to_value(schema).unwrap()
}

#[test]
fn test_struct_schema() {
  assert_eq!(Struct::name(), "Struct");
  assert_eq!(
    to_json(Struct::schema()),
    json!({
      "type": "object",
      "properties": {
        "@context": { "examples": [{ "ex": "http://example.com/" }] },
        "@type": { "type": "string", "enum": ["ex:Struct"] },
        "@id": {
          "type": "string",
          "pattern": "^urn:uuid:[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-5][0-9a-fA-F]{3}-[089abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },
        "ex:field_0": { "type": "string" },
        "ex:field_1": { "type": "integer", "format": "int64", "minimum": 0 },
        "ex:field_2": { "type": "array", "items": { "$ref": "#/components/schemas/SubStruct" } },
        "ex:field_3": { "$ref": "#/components/schemas/Color" },
        "ex:field_4": { "$ref": "#/components/schemas/Value" },
      },
      "required": ["@type", "@id", "ex:field_0", "ex:field_2", "ex:field_3"],
      "x-jsonld-type": "http://example.com/Struct",
      "x-jsonld-predicates": {
        "ex:field_0": "http://example.com/field_0",
        "ex:field_1": "http://example.com/field_1",
        "ex:field_2": "http://example.com/field_2",
        "ex:field_3": "http://example.com/field_3",
        "ex:field_4": "http://example.com/field_4",
      },
    })
  );
}

#[test]
fn test_longest_prefix_and_default_id() {
  let schema = to_json(SubStruct::schema());

  assert_eq!(
    schema["properties"]["@type"]["enum"],
    json!(["sub:SubStruct"])
  );
  assert_eq!(schema["properties"]["@id"], json!({ "type": "string" }));
  assert_eq!(
    schema["properties"]["@context"]["examples"],
    json!([{ "ex": "http://example.com/", "sub": "http://example.com/sub/" }])
  );
  assert_eq!(
    schema["properties"]["http://other.org/sub_field_1"],
    json!({ "$ref": "#/components/schemas/SubStruct" })
  );
  assert_eq!(schema["required"], json!(["@type", "sub:sub_field_0"]));
}

#[test]
fn test_enum_schemas() {
  assert_eq!(
    to_json(Color::schema()),
    json!({
      "type": "object",
      "properties": {
        "@id": { "type": "string", "enum": ["ex:Red", "ex:Blue"] },
      },
      "required": ["@id"],
    })
  );

  assert_eq!(
    to_json(Value::schema()),
    json!({
      "oneOf": [
        {
          "type": "object",
          "properties": { "ex:text": { "type": "string" } },
          "required": ["ex:text"],
        },
        {
          "type": "object",
          "properties": {
            "ex:value": {
              "type": "object",
              "properties": { "ex:number": { "type": "integer", "format": "int64" } },
              "required": ["ex:number"],
            },
          },
          "required": ["ex:value"],
        },
      ],
    })
  );
}

#[test]
fn test_openapi_components() {
  #[derive(OpenApi)]
  #[openapi(components(schemas(Struct)))]
  struct ApiDoc;

  let openapi = serde_json::to_value(ApiDoc::openapi()).unwrap();
  let mut names = openapi["components"]["schemas"]
    .as_object()
    .unwrap()
    .keys()
    .cloned()
    .collect::<Vec<_>>();
  names.sort();

  assert_eq!(names, ["Color", "Struct", "SubStruct", "Value"]);
}
