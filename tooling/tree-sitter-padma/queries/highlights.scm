[
  "let"
  "ধরি"
  "print"
  "দেখাও"
  "if"
  "যদি"
  "else"
  "নইলে"
  "while"
  "যতক্ষণ"
  "for"
  "প্রতি"
  "in"
  "মধ্যে"
  "function"
  "fn"
  "ফাংশন"
  "return"
  "ফেরত"
  "import"
  "ইমপোর্ট"
  "export"
  "রপ্তানি"
  "as"
  "হিসেবে"
] @keyword

[(boolean) (null)] @constant.builtin
(number) @number
(string) @string
(comment) @comment

(function_definition name: (identifier) @function)
(call_expression function: (identifier) @function.call)
(call_expression function: (qualified_identifier) @function.call)
(parameter_list (identifier) @variable.parameter)
(let_declaration name: (identifier) @variable)
(for_statement name: (identifier) @variable)
(import_statement alias: (identifier) @namespace)
(qualified_identifier (identifier) @variable)

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ":" "."] @punctuation.delimiter
["=" "==" "!=" "<" "<=" ">" ">=" "+" "-" "*" "/"] @operator
