use crate::Target;
use jtd::form::{self, TypeValue};
use jtd::{Form, Schema};
use std::collections::BTreeMap;
use teeter_inflector::string::singularize::to_singular;

#[derive(Debug)]
pub enum Ast {
    Ref(String),
    Boolean,
    String,
    Timestamp,
    ArrayOf(Box<Ast>),
    NullableOf(Box<Ast>),
    Alias(Alias),
    Enum(Enum),
    Struct(Struct),
    Discriminator(Discriminator),
}

#[derive(Debug)]
pub struct Alias {
    pub name: String,
    pub description: String,
    pub type_: Box<Ast>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub description: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub description: String,
    pub json_value: String,
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub description: String,
    pub has_additional: bool,
    pub fields: Vec<StructField>,
}

#[derive(Debug)]
pub struct StructField {
    pub name: String,
    pub json_name: String,
    pub description: String,
    pub optional: bool,
    pub type_: Ast,
}

#[derive(Debug)]
pub struct Discriminator {
    pub name: String,
    pub description: String,
    pub tag_name: String,
    pub tag_json_name: String,
    pub variants: BTreeMap<String, DiscriminatorVariant>, // key is tag value
}

#[derive(Debug)]
pub struct DiscriminatorVariant {
    pub name: String,
    pub description: String,
    pub tag_name: String,
    pub tag_json_name: String,
    pub tag_json_value: String,
    pub fields: Vec<StructField>,
}

pub fn from_schema<T: Target>(
    target: &T,
    root_name: String,
    schema: &Schema,
) -> (Ast, BTreeMap<String, Ast>) {
    let root = from_schema_top_level(target, root_name, schema);
    let definitions = schema
        .definitions
        .iter()
        .map(|(name, sub_schema)| {
            (
                name.clone(),
                from_schema_top_level(target, name.clone(), sub_schema),
            )
        })
        .collect();

    (root, definitions)
}

fn from_schema_top_level<T: Target>(target: &T, name: String, schema: &Schema) -> Ast {
    let ast = _from_schema(target, &mut vec![name.clone()], schema);
    match ast {
        Ast::Alias(_) | Ast::Enum(_) | Ast::Struct(_) | Ast::Discriminator(_) => ast,
        _ => Ast::Alias(Alias {
            name: T::name_type(&[name]),
            description: "".into(),
            type_: Box::new(ast),
        }),
    }
}

fn _from_schema<T: Target>(target: &T, path: &mut Vec<String>, schema: &Schema) -> Ast {
    match schema.form {
        Form::Ref(form::Ref {
            ref definition,
            nullable,
        }) => with_nullable(target, nullable, Ast::Ref(definition.clone())),
        Form::Type(form::Type {
            ref type_value,
            nullable,
        }) => with_nullable(
            target,
            nullable,
            match type_value {
                TypeValue::Boolean => Ast::Boolean,
                TypeValue::String => Ast::String,
                TypeValue::Timestamp => Ast::Timestamp,
                _ => todo!(),
            },
        ),
        Form::Enum(form::Enum {
            ref values,
            nullable,
        }) => {
            let mut variants = vec![];
            for value in values {
                path.push(value.into());
                variants.push(EnumVariant {
                    name: T::name_enum_variant(path),
                    description: "".into(),
                    json_value: value.into(),
                });
                path.pop();
            }

            let name = T::name_type(path);
            with_nullable(
                target,
                nullable,
                Ast::Enum(Enum {
                    name,
                    description: "".into(),
                    variants,
                }),
            )
        }
        Form::Elements(form::Elements {
            ref schema,
            nullable,
        }) => {
            // singularize the last path segment, because it turns out it was
            // referring to a list of things, and thus we presume the name may
            // be in the plural.
            let last = path.pop().expect("todo: top-level elements");
            path.push(to_singular(&last));

            with_nullable(
                target,
                nullable,
                Ast::ArrayOf(Box::new(_from_schema(target, path, schema))),
            )
        }
        Form::Properties(form::Properties {
            ref required,
            ref optional,
            additional,
            nullable,
            ..
        }) => {
            let struct_name = T::name_type(path);

            let mut fields = Vec::new();
            for (name, sub_schema) in required {
                // Determine the field's name.
                path.push(name.clone());
                let field_name = T::name_field(path);
                path.pop();

                // Generate an expr representing sub_schema
                path.push(name.clone());
                let field_ast = _from_schema(target, path, sub_schema);
                path.pop();

                // Add the expr to the set of fields for the struct
                fields.push(StructField {
                    name: field_name,
                    json_name: name.clone(),
                    description: "".into(),
                    optional: false,
                    type_: field_ast,
                });
            }

            with_nullable(
                target,
                nullable,
                Ast::Struct(Struct {
                    name: struct_name,
                    description: "".into(),
                    has_additional: additional,
                    fields,
                }),
            )
        }
        Form::Discriminator(form::Discriminator {
            ref discriminator,
            ref mapping,
            nullable,
        }) => {
            let discriminator_name = T::name_type(path);

            path.push(discriminator.clone());
            let tag_name = T::name_field(path);
            path.pop();

            let mut variants = BTreeMap::new();
            for (tag_value, sub_schema) in mapping {
                path.push(tag_value.clone());
                let sub_expr = _from_schema(target, path, sub_schema);
                path.pop();

                // We know we are returning a struct here, because
                // sub_schema must be of the properties form.
                let struct_ = match sub_expr {
                    Ast::Struct(s) => s,
                    _ => unreachable!(),
                };

                variants.insert(
                    tag_value.clone(),
                    DiscriminatorVariant {
                        name: struct_.name,
                        description: struct_.description,
                        tag_name: tag_name.clone(),
                        tag_json_name: discriminator.clone(),
                        tag_json_value: tag_value.clone(),
                        fields: struct_.fields,
                    },
                );
            }

            with_nullable(
                target,
                nullable,
                Ast::Discriminator(Discriminator {
                    name: discriminator_name,
                    tag_name: tag_name.clone(),
                    tag_json_name: discriminator.clone(),
                    description: "".into(),
                    variants,
                }),
            )
        }
        _ => todo!(),
    }
}

fn with_nullable<T: Target>(target: &T, nullable: bool, ast: Ast) -> Ast {
    // We need to wrap ast in NullableOf if the caller passed in nullable
    // and if ast isn't already nullable to begin with.
    let needs_nullable = nullable
        && match ast {
            Ast::Boolean => !T::booleans_are_nullable(),
            Ast::String => !T::strings_are_nullable(),
            Ast::Timestamp => !T::timestamps_are_nullable(),
            Ast::ArrayOf(_) => !T::arrays_are_nullable(),
            Ast::Alias(_) => !T::aliases_are_nullable(),
            Ast::Enum(_) => !T::enums_are_nullable(),
            Ast::Struct(_) => !T::structs_are_nullable(),
            Ast::Discriminator(_) => !T::discriminators_are_nullable(),
            Ast::Ref(_) => true,
            Ast::NullableOf(_) => false,
        };

    if needs_nullable {
        Ast::NullableOf(Box::new(ast))
    } else {
        ast
    }
}