const PREC = {
  OR: 1,
  AND: 2,
  EQUALITY: 3,
  COMPARISON: 4,
  TERM: 5,
  FACTOR: 6,
  UNARY: 7,
  POSTFIX: 8,
};

export default grammar({
  name: 'padma',

  extras: $ => [/[ \t\r\f]/, $.comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat(choice($._statement, '\n')),

    _statement: $ => choice(
      $.let_declaration,
      $.print_statement,
      $.assignment_statement,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.function_definition,
      $.return_statement,
      $.import_statement,
      $.export_statement,
      $.expression_statement,
    ),

    let_declaration: $ => seq(
      choice('let', 'ধরি'),
      field('name', $.identifier),
      '=',
      field('value', $.expression),
    ),

    print_statement: $ => seq(choice('print', 'দেখাও'), field('value', $.expression)),

    assignment_statement: $ => seq(
      field('name', choice($.identifier, $.qualified_identifier)),
      '=',
      field('value', $.expression),
    ),

    if_statement: $ => seq(
      choice('if', 'যদি'),
      field('condition', $.expression),
      field('consequence', $.block),
      optional(seq(choice('else', 'নইলে'), field('alternative', $.block))),
    ),

    while_statement: $ => seq(
      choice('while', 'যতক্ষণ'),
      field('condition', $.expression),
      field('body', $.block),
    ),

    for_statement: $ => seq(
      choice('for', 'প্রতি'),
      field('name', $.identifier),
      choice('in', 'মধ্যে'),
      field('collection', $.expression),
      field('body', $.block),
    ),

    function_definition: $ => seq(
      choice('function', 'fn', 'ফাংশন'),
      field('name', $.identifier),
      $.parameter_list,
      field('body', $.block),
    ),

    parameter_list: $ => seq('(', optional(commaSep1($.identifier)), ')'),

    return_statement: $ => prec.right(seq(
      choice('return', 'ফেরত'),
      optional($.expression),
    )),

    import_statement: $ => seq(
      choice('import', 'ইমপোর্ট'),
      field('path', $.string),
      optional(seq(choice('as', 'হিসেবে'), field('alias', $.identifier))),
    ),

    export_statement: $ => seq(
      choice('export', 'রপ্তানি'),
      choice($.let_declaration, $.function_definition),
    ),

    expression_statement: $ => $.expression,

    block: $ => seq('{', repeat(choice($._statement, '\n')), '}'),

    expression: $ => choice(
      $.identifier,
      $.qualified_identifier,
      $.number,
      $.string,
      $.boolean,
      $.null,
      $.list,
      $.map,
      $.parenthesized_expression,
      $.call_expression,
      $.index_expression,
      $.slice_expression,
      $.unary_expression,
      $.binary_expression,
    ),

    parenthesized_expression: $ => seq('(', $.expression, ')'),

    unary_expression: $ => prec(PREC.UNARY, seq('-', $.expression)),

    binary_expression: $ => choice(
      prec.left(PREC.OR, seq($.expression, '||', $.expression)),
      prec.left(PREC.AND, seq($.expression, '&&', $.expression)),
      prec.left(PREC.EQUALITY, seq($.expression, choice('==', '!='), $.expression)),
      prec.left(PREC.COMPARISON, seq($.expression, choice('<', '<=', '>', '>='), $.expression)),
      prec.left(PREC.TERM, seq($.expression, choice('+', '-'), $.expression)),
      prec.left(PREC.FACTOR, seq($.expression, choice('*', '/'), $.expression)),
    ),

    call_expression: $ => prec(PREC.POSTFIX, seq(
      field('function', choice($.identifier, $.qualified_identifier)),
      '(',
      optional(commaSep1($.expression)),
      ')',
    )),

    index_expression: $ => prec.left(PREC.POSTFIX, seq(
      field('target', $.expression),
      '[',
      field('index', $.expression),
      ']',
    )),

    slice_expression: $ => prec.left(PREC.POSTFIX, seq(
      field('target', $.expression),
      '[',
      optional(field('start', $.expression)),
      ':',
      optional(field('end', $.expression)),
      ']',
    )),

    list: $ => seq('[', optional(commaSep1($.expression)), ']'),

    map: $ => seq(
      '{',
      optional(commaSep1(seq(field('key', $.expression), ':', field('value', $.expression)))),
      '}',
    ),

    qualified_identifier: $ => seq($.identifier, '.', $.identifier),
    identifier: _ => token(/[A-Za-z_\u0980-\u09E5\u09F0-\u09FF][A-Za-z0-9_\u0980-\u09FF]*/),
    number: _ => token(choice(/[0-9]+(\.[0-9]+)?/, /[০-৯]+(\.[০-৯]+)?/)),
    string: _ => token(seq('"', repeat(choice(/[^"\\\n]+/, /\\./)), '"')),
    boolean: _ => choice('true', 'false', 'সত্য', 'মিথ্যা'),
    null: _ => choice('none', 'কিছুইনা'),
    comment: _ => token(seq('#', /[^\n]*/)),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}
