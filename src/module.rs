use crate::{
    components::{
        Code, Data, Elem, Export, Func, Funcidx, Functype, Global, Import, Memtype, Tabletype,
        Typeidx,
    },
    decode::Decode,
    execute::ExecuteError,
    reader::Reader,
    vector::Vector,
    DecodeError, ModuleInstance, ModuleInstanceOptions, VectorFactory,
};
use core::fmt::{Debug, Formatter};

const SECTION_ID_CUSTOM: u8 = 0;
const SECTION_ID_TYPE: u8 = 1;
const SECTION_ID_IMPORT: u8 = 2;
const SECTION_ID_FUNCTION: u8 = 3;
const SECTION_ID_TABLE: u8 = 4;
const SECTION_ID_MEMORY: u8 = 5;
const SECTION_ID_GLOBAL: u8 = 6;
const SECTION_ID_EXPORT: u8 = 7;
const SECTION_ID_START: u8 = 8;
const SECTION_ID_ELEMENT: u8 = 9;
const SECTION_ID_CODE: u8 = 10;
const SECTION_ID_DATA: u8 = 11;

pub struct Module<V: VectorFactory> {
    types: V::Vector<Functype<V>>,
    funcs: V::Vector<Func<V>>,
    tables: V::Vector<Tabletype>,
    imports: V::Vector<Import<V>>,
    mem: Option<Memtype>,
    globals: V::Vector<Global<V>>,
    elem: V::Vector<Elem<V>>,
    data: V::Vector<Data<V>>,
    start: Option<Funcidx>,
    exports: V::Vector<Export<V>>,
}

impl<V: VectorFactory> Module<V> {
    pub fn decode(wasm_bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut this = Self {
            types: V::create_vector(None),
            funcs: V::create_vector(None),
            tables: V::create_vector(None),
            mem: None,
            globals: V::create_vector(None),
            elem: V::create_vector(None),
            data: V::create_vector(None),
            start: None,
            imports: V::create_vector(None),
            exports: V::create_vector(None),
        };
        let mut reader = Reader::new(wasm_bytes);

        // Preamble
        let _ = Magic::decode(&mut reader)?;
        let _ = Version::decode(&mut reader)?;

        // Sections
        this.decode_sections(&mut reader)?;

        Ok(this)
    }

    fn decode_sections(&mut self, reader: &mut Reader) -> Result<(), DecodeError> {
        let mut last_section_id = SECTION_ID_CUSTOM;
        let mut function_section: V::Vector<Typeidx> = V::create_vector(None);
        while !reader.is_empty() {
            let section_id = reader.read_u8()?;
            let section_size = reader.read_u32()? as usize;
            let mut section_reader = Reader::new(reader.read(section_size)?);

            if section_id == SECTION_ID_CUSTOM {
                continue;
            }

            if section_id < last_section_id {
                return Err(DecodeError::InvalidSectionOrder {
                    current_section_id: section_id as u8,
                    last_section_id: last_section_id as u8,
                });
            }

            match section_id {
                SECTION_ID_TYPE => {
                    self.types = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SECTION_ID_IMPORT => {
                    self.imports = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SECTION_ID_FUNCTION => {
                    function_section = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SECTION_ID_TABLE => {
                    self.tables = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SECTION_ID_MEMORY => {
                    let value = section_reader.read_u32()? as usize;
                    if value > 1 {
                        return Err(DecodeError::InvalidMemoryCount { value });
                    }
                    if value == 1 {
                        let mem = Memtype::decode(&mut section_reader)?;
                        self.mem = Some(mem);
                    }
                }
                SECTION_ID_GLOBAL => {
                    self.globals = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SECTION_ID_EXPORT => {
                    self.exports = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SECTION_ID_START => {
                    self.start = Some(Decode::decode(&mut section_reader)?);
                }
                SECTION_ID_ELEMENT => {
                    self.elem = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                SECTION_ID_CODE => {
                    let code_section: V::Vector<Code<V>> =
                        Decode::decode_vector::<V>(&mut section_reader)?;
                    if function_section.len() != code_section.len() {
                        return Err(DecodeError::MismatchFunctionAndCodeSectionSize {
                            function_section_size: function_section.len(),
                            code_section_size: code_section.len(),
                        });
                    }
                    self.funcs = V::create_vector(Some(function_section.len()));
                    for (&ty, code) in function_section.iter().zip(code_section.iter()) {
                        self.funcs.push(Func {
                            ty,
                            locals: V::clone_vector(&code.locals),
                            body: code.body.clone(),
                        });
                    }
                }
                SECTION_ID_DATA => {
                    self.data = Decode::decode_vector::<V>(&mut section_reader)?;
                }
                _ => {
                    return Err(DecodeError::InvalidSectionId { value: section_id });
                }
            }
            last_section_id = section_id;

            if !section_reader.is_empty() {
                return Err(DecodeError::InvalidSectionByteSize {
                    section_id: section_id as u8,
                    expected_byte_size: section_size,
                    actual_byte_size: section_reader.position(),
                });
            }
        }
        Ok(())
    }

    pub fn instantiate(
        self,
        options: ModuleInstanceOptions<V>,
    ) -> Result<ModuleInstance<V>, ExecuteError> {
        // TODO: validate
        let instance = ModuleInstance::new(self, options)?;
        Ok(instance)
    }

    pub fn types(&self) -> &[Functype<V>] {
        &self.types
    }

    pub fn funcs(&self) -> &[Func<V>] {
        &self.funcs
    }

    pub fn tables(&self) -> &[Tabletype] {
        &self.tables
    }

    pub fn mem(&self) -> Option<Memtype> {
        self.mem
    }

    pub fn globals(&self) -> &[Global<V>] {
        &self.globals
    }

    pub fn elem(&self) -> &[Elem<V>] {
        &self.elem
    }

    pub fn data(&self) -> &[Data<V>] {
        &self.data
    }

    pub fn start(&self) -> Option<Funcidx> {
        self.start
    }

    pub fn imports(&self) -> &[Import<V>] {
        &self.imports
    }

    pub fn exports(&self) -> &[Export<V>] {
        &self.exports
    }
}

impl<V: VectorFactory> Debug for Module<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Module")
            .field("types", &self.types.as_ref())
            .field("funcs", &self.funcs.as_ref())
            .field("tables", &self.tables.as_ref())
            .field("mem", &self.mem)
            .field("globals", &self.globals.as_ref())
            .field("elem", &self.elem.as_ref())
            .field("data", &self.data.as_ref())
            .field("start", &self.start)
            .field("imports", &self.imports.as_ref())
            .field("exports", &self.exports.as_ref())
            .finish()
    }
}

impl<V: VectorFactory> Clone for Module<V> {
    fn clone(&self) -> Self {
        Self {
            types: V::clone_vector(&self.types),
            funcs: V::clone_vector(&self.funcs),
            tables: V::clone_vector(&self.tables),
            mem: self.mem,
            globals: V::clone_vector(&self.globals),
            elem: V::clone_vector(&self.elem),
            data: V::clone_vector(&self.data),
            start: self.start,
            imports: V::clone_vector(&self.imports),
            exports: V::clone_vector(&self.exports),
        }
    }
}

struct Magic;

impl Magic {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let mut value = [0; 4];
        reader.read_exact(&mut value)?;
        if value != *b"\0asm" {
            return Err(DecodeError::InvalidMagic { value });
        }
        Ok(Self)
    }
}

struct Version;

impl Version {
    fn decode(reader: &mut Reader) -> Result<Self, DecodeError> {
        let mut value = [0; 4];
        reader.read_exact(&mut value)?;
        if value != [1, 0, 0, 0] {
            return Err(DecodeError::InvalidVersion { value });
        }
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StdVectorFactory;

    fn decode(wasm: &[u8]) -> Module<StdVectorFactory> {
        Module::decode(wasm).expect("decode module")
    }

    #[test]
    fn decode_empty_module() {
        // (module)
        let input = [0, 97, 115, 109, 1, 0, 0, 0];
        decode(&input);
    }

    #[test]
    fn decode_add_two() {
        // (module
        //   (func (export "addTwo") (param i32 i32) (result i32)
        //     local.get 0
        //     local.get 1
        //     i32.add))
        let input = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 7, 1, 96, 2, 127, 127, 1, 127, 3, 2, 1, 0, 7, 10, 1, 6,
            97, 100, 100, 84, 119, 111, 0, 0, 10, 9, 1, 7, 0, 32, 0, 32, 1, 106, 11,
        ];
        let module = decode(&input);
        assert_eq!(1, module.exports().len());
        assert_eq!("addTwo", module.exports()[0].name.as_str());
    }
}
