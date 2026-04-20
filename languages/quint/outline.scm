(module_definition
  "module" @context
  (qualified_identifier) @name) @item

(operator_definition
  name: (qualified_identifier) @name) @item

(type_alias
  "type" @context
  (qualified_identifier) @name) @item

(constant_declaration
  "const" @context
  (qualified_identifier) @name) @item

(variable_definition
  "var" @context
  (qualified_identifier) @name) @item

(assumption
  "assume" @context
  (qualified_identifier) @name) @item
