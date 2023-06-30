use std::{cell::{RefCell, RefMut}, fmt::Display};

peg::parser!(
    pub grammar class_parser<'a>() for [u8]{

        rule u8() -> u8
            = x:[_] {x}

        rule u16() -> u16
            = v:$([_][_]) {unsafe{u16::from_be_bytes([*v.get_unchecked(0), *v.get_unchecked(1)])}}

        rule u32() -> u32
            = v:$([_][_][_][_]){ unsafe { u32::from_be_bytes([*v.get_unchecked(0), *v.get_unchecked(1), *v.get_unchecked(2), *v.get_unchecked(3)])} }
        rule u64() -> u64
            = v:$([_][_][_][_][_][_][_][_])
            { unsafe { 
                u64::from_be_bytes(
                    [*v.get_unchecked(0), *v.get_unchecked(1), *v.get_unchecked(2), *v.get_unchecked(3),
                     *v.get_unchecked(4), *v.get_unchecked(5), *v.get_unchecked(6), *v.get_unchecked(7)])} }
            
        rule f32() -> f32
            = v:u32() {f32::from_bits(v)}
        
        rule f64() -> f64
            = v:u64() {f64::from_bits(v)}
        
        rule slice(len: usize) -> &'input[u8]
            = v:$([_]*<{len}>) {v}

        rule list<T>(item: rule<T>) -> Vec<T>
        = l:u16() out:item()*<{l as usize}> {out}

        rule magic()
            = quiet!{[0xca][0xfe][0xba][0xbe]}
            / expected!("magic number")

        rule version() -> (u16, u16)
        = minor:u16() major:u16() {(minor, major)}

        rule utf_info() -> ConstantPoolEntry<'input>
            = length:u16() bytes:slice(length as usize) {ConstantPoolEntry::Utf8Info(bytes)}
        
        rule integer_info() -> ConstantPoolEntry<'input>
            = value:u32() {ConstantPoolEntry::IntegerInfo(value as i32)}
        
        rule float_info() -> ConstantPoolEntry<'input>
            = value:f32() {ConstantPoolEntry::FloatInfo(value)}

        rule long_info() -> ConstantPoolEntry<'input>
        = value:u64() {ConstantPoolEntry::LongInfo(value as i64)}

        rule double_info() -> ConstantPoolEntry<'input>
        = value:f64() {ConstantPoolEntry::DoubleInfo(value)}

        rule class_info() ->  ConstantPoolEntry<'input>
            = index:u16()                     {ConstantPoolEntry::ClassInfo(index)}

        rule string_info() ->  ConstantPoolEntry<'input>
        = index:u16() {ConstantPoolEntry::StringInfo(index)}


        rule name_and_type_info() -> ConstantPoolEntry<'input>
            = name:u16() descriptor:u16()     {ConstantPoolEntry::NameAndTypeInfo { name, descriptor }}
        
        rule field_ref_info() -> ConstantPoolEntry<'input>
            = class:u16() name_and_type:u16() {ConstantPoolEntry::FieldRef { class, name_and_type }}

        rule method_ref_info() -> ConstantPoolEntry<'input>
            = class:u16() name_and_type:u16() {ConstantPoolEntry::MethodRef { class, name_and_type }}

        rule interface_ref_info() -> ConstantPoolEntry<'input>
            = class:u16() name_and_type:u16() {ConstantPoolEntry::InterfaceMethodRef { class, name_and_type }}
        
        rule method_handle_info() -> ConstantPoolEntry<'input>
            = kind:u8() index:u16() {ConstantPoolEntry::MethodHandle{reference_kind: kind, reference_index: index}}
        
        rule method_type_info() -> ConstantPoolEntry<'input>
            = index:u16() {ConstantPoolEntry::MethodType { descriptor_index: index }}
        
        rule dynamic_info() -> ConstantPoolEntry<'input>
            = attr_index: u16() name_index: u16() {ConstantPoolEntry::DynamicInfo { bootstrap_method_attr_index: attr_index, name_and_type_index: name_index }}

        rule invoke_dynamic_info() -> ConstantPoolEntry<'input>
            = attr_index: u16() name_index: u16() {ConstantPoolEntry::InvokeDynamicInfo { bootstrap_method_attr_index: attr_index, name_and_type_index: name_index }}
        
        rule module_info() -> ConstantPoolEntry<'input>
            = name_index: u16() {ConstantPoolEntry::ModuleInfo { name_index }}

        rule package_info() -> ConstantPoolEntry<'input>
            = name_index: u16() {ConstantPoolEntry::PackageInfo { name_index }}

        rule constant_pool_entry(count: &mut RefMut<u16>) -> ConstantPoolEntry<'input>
            = [1]  value:utf_info()            {value}
            / [3]  value:integer_info()        {value}
            / [4]  value:float_info()          {value}
            / [5]  value:long_info()           {**count -= 1; value}
            / [6]  value:double_info()         {**count -= 1; value}
            / [7]  value:class_info()          {value}
            / [8]  value:string_info()         {value}
            / [9]  value:field_ref_info()      {value}
            / [10] value:method_ref_info()     {value}
            / [11] value:interface_ref_info()  {value}
            / [12] value:name_and_type_info()  {value}
            / [15] value:method_handle_info()  {value}
            / [16] value:method_type_info()    {value}
            / [17] value:dynamic_info()        {value}
            / [18] value:invoke_dynamic_info() {value}
            / [19] value:module_info()         {value}
            / [20] value:package_info()        {value}
            / x:u8()                           {panic!("No item with tag {}",x)}

        rule constant_pool(count: &mut RefMut<u16>) -> Vec<ConstantPoolEntry<'input>>
            = entries:constant_pool_entry(count)*<{(**count-1) as usize}> {entries}

        rule attribute() -> Attribute<'input>
        = name_index:u16()
          length:u32()
          bytes:slice(length as usize)
          {
            Attribute{
                name_index,
                bytes
            }
          }
        
        rule field() -> Field<'input>
        = access_flags:u16()
          name_index:u16()
          descriptor_index:u16()
          attributes:list(<attribute()>)
          {
            Field{
                access_flags,
                name_index,
                descriptor_index,
                attributes
            }
          }

        rule method() -> Method<'input>
        = access_flags:u16()
          name_index:u16()
          descriptor_index:u16()
          attributes:list(<attribute()>)
          {
            Method{
                access_flags,
                name_index,
                descriptor_index,
                attributes
            }
          }



        #[no_eof]
        pub rule class_file() -> ClassFile<'input>
            = magic() 
              version:version()
              constant_pool_count: (cp:u16() {RefCell::new(cp)})
              constant_pool: constant_pool(&mut constant_pool_count.borrow_mut())
              access_flags:u16()
              this_class:u16()
              super_class:u16()
              interfaces:list(<u16()>)
              fields:list(<field()>)
              methods:list(<method()>)
              attributes:list(<attribute()>)
              {
                ClassFile{
                    constant_pool,
                    version,
                    access_flags,
                    super_class,
                    this_class,
                    interfaces,
                    fields,
                    methods,
                    attributes
                }
              }
    }
);

#[derive(Debug)]
pub enum ConstantPoolEntry<'a>{
    Utf8Info(&'a [u8]),
    ClassInfo(u16),
    FieldRef{
        class: u16,
        name_and_type: u16
    },
    MethodRef{
        class: u16,
        name_and_type: u16
    },
    InterfaceMethodRef{
        class: u16,
        name_and_type: u16
    },
    StringInfo(u16),
    IntegerInfo(i32),
    FloatInfo(f32),
    LongInfo(i64),
    DoubleInfo(f64),
    NameAndTypeInfo{
        name: u16,
        descriptor: u16
    },
    MethodHandle{
        reference_kind: u8,
        reference_index: u16
    },
    MethodType{
        descriptor_index: u16
    },
    DynamicInfo{
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16
    },
    InvokeDynamicInfo{
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16
    },
    ModuleInfo{
        name_index: u16
    },
    PackageInfo{
        name_index: u16
    }
}

#[derive(Debug)]
pub struct Attribute<'a>{
    pub name_index: u16,
    pub bytes: &'a [u8]
}

#[derive(Debug)]
pub struct Field<'a>{
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute<'a>>
}

#[derive(Debug)]
pub struct Method<'a>{
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute<'a>>
}

#[derive(Debug)]
pub struct ClassFile<'a>{
    pub version: (u16, u16),
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub constant_pool: Vec<ConstantPoolEntry<'a>>,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field<'a>>,
    pub methods: Vec<Method<'a>>,
    pub attributes: Vec<Attribute<'a>>
}

impl<'a> Display for ClassFile<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({} {})", self.version.0, self.version.1))?;
        f.write_fmt(format_args!(" constants: {}",self.constant_pool.len()))?;
        f.write_fmt(format_args!(" interfaces: {}",self.interfaces.len()))?;
        f.write_fmt(format_args!(" fields: {}",self.fields.len()))?;
        f.write_fmt(format_args!(" methods: {}",self.methods.len()))?;
        f.write_fmt(format_args!(" class attributes: {}",self.attributes.len()))?;
        Ok(())
    }
}