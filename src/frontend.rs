use peg::parser;

#[derive(Debug)]
pub enum Type<'a>{
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    Pointer(&'a str)
}

#[derive(Debug)]
pub enum Expression<'a>{
    Method{
        name: &'a str,
        arguments: Vec<Expression<'a>>
    },
    Numeric(&'a str),
    Block{
        params: Vec<(&'a str, Type<'a>)>,
        statements: Vec<Statement<'a>>
    },
    Identifier(&'a str)
}

#[derive(Debug)]
pub struct Statement<'a>{
    pub output: Vec<&'a str>,
    pub expression: Expression<'a>
}

#[derive(Debug)]
pub struct Function<'a>{
    pub exported: bool,
    pub name: &'a str,
    pub params: Vec<(&'a str, Type<'a>)>,
    pub returns: Vec<Type<'a>>,
    pub statements: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub struct Import<'a>{
    path: Vec<&'a str>,
    imports: Vec<&'a str>
}

#[derive(Debug)]
pub struct Module<'a>{
    imports: Vec<Import<'a>>,
    functions: Vec<Function<'a>>,
    exports: Vec<&'a str>
}

enum ModuleEntry<'a>{
    Func(Function<'a>),
    Import(Import<'a>)
}

parser!(
    pub grammar file_parser() for str{

        rule eol() = (_ "\n" _)+

        rule __() = quiet!{_ ("\n" _)*}

        rule _() = quiet!{[' ' | '\t' | '\r']*}

        rule identifier() -> &'input str
            = n:quiet!{$(['a'..='z' | 'A'..='Z' | '#' | '_' | '$']['a'..='z' | 'A'..='Z' | '#' | '_' | '$' | '0'..='9']*)} {n}
            / expected!("identifier")

        rule identifier_list() -> Vec<&'input str>
            = names: ( (_ n:identifier() _ {n}) ** ","){names}

        rule type_() -> Type<'input>
            = "i8"                       {Type::I8}
            / "u8"                       {Type::U8}
            / "i16"                      {Type::I16}
            / "u16"                      {Type::U16}
            / "i32"                      {Type::I32}
            / "u32"                      {Type::U32}
            / "i64"                      {Type::I64}
            / "u64"                      {Type::U64}
            / "f32"                      {Type::F32}
            / "f64"                      {Type::F64}
            / "@<" name:identifier() ">" { Type::Pointer(name) }
        
        rule parameter_list() -> Vec<(&'input str, Type<'input>)>
        = "(" _ params:((_ name:identifier() _ ":" _ ty:type_() {(name, ty)}) ** ",") _ ")" {params}



        rule expression() -> Expression<'input>
        = &"(" params: parameter_list() __ "{" __ statements:statement_list() __ "}"  { Expression::Block { params, statements } }
        / &identifier() name:identifier() _ arguments:("(" __ arguments:expression_list() __ ")"{arguments})? { 
            match arguments {
                Some(arguments) => Expression::Method { name, arguments },
                _               => Expression::Identifier(name) 
            }
        }

        rule expression_list() -> Vec<Expression<'input>>
        = n:( (__ n:expression() __ {n}) ** ","){n}

        rule statement() -> Statement<'input>
        = _ output:(n:identifier_list() _ "=" {n})? _ expression:expression() eol()
        {
            Statement { 
                output: output.unwrap_or_else(||Vec::with_capacity(0)),
                expression
            }
        }

        rule statement_list() -> Vec<Statement<'input>>
        = out:(_ s:statement() _ {s})* {out}

        rule function() -> Function<'input>
            = exported:"export"? _ "func" _ name:identifier() _ "(" params:((_ name:identifier() _ ":" _ ty:type_() {(name, ty)}) ** ",") _ ")" __ returns:("->" _ ret:((_ ty:type_() _ {ty}) ** ",") {ret})? __
            "{" __ statements:statement_list() __ "}"
            {
                Function{
                    exported: exported.is_some(),
                    name,
                    params,
                    returns: returns.unwrap_or_else(|| Vec::with_capacity(0)),
                    statements
                }
            }
        
        rule path() -> Vec<&'input str>
            = s:(identifier() ** "/"){ s}
        
        rule import() -> Import<'input>
            = "from" _ path:path() _ "use" _ imports: ( (_ n:identifier() _ {n}) ** ",") ","? eol()
            {
                Import{
                    path,
                    imports
                }
            }
        
        rule module_entry() -> ModuleEntry<'input>
        = &("export"? _ "func") f:function() {ModuleEntry::Func(f)}
        / &("from") i:import() {ModuleEntry::Import(i) }

        pub rule module() -> Module<'input>
            = __ entries:(__ entry:module_entry() __ {entry})*
            {
                let mut imports = Vec::new();
                let mut functions = Vec::new();
                let mut exports = Vec::new();
                for entry in entries{
                    match entry{
                        ModuleEntry::Func(f) => {
                            if f.exported{
                                exports.push(f.name);
                            }
                            functions.push(f);
                            
                        }
                        ModuleEntry::Import(i) => {
                            imports.push(i)
                        }
                    }
                }

                Module{
                    functions,
                    exports,
                    imports
                }
            }
    }
);