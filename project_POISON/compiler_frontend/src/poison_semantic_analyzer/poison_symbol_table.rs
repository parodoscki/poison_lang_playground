use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Void,
    I64,
    I32,
    I16,
    I8,

    F64,
    F32,
    F16,
    F8,

    U64,
    U32,
    U16,
    U8,
    Bool,
    Str,
    Custom(IdentifierId),
}

impl Type {
    pub fn to_backend_id(&self) -> u32 {
        match self {
            Type::Void => 0,
            Type::I64  => 1,
            Type::I32  => 2,
            Type::I16  => 3,
            Type::I8   => 4,
            
            Type::F64  => 5,
            Type::F32  => 6,
            Type::F16  => 7,
            Type::F8   => 8,
            
            Type::U64  => 1,
            Type::U32  => 2,
            Type::U16  => 3,
            Type::U8   => 4,
            
            Type::Bool => 5,
            
            Type::Str  => 9,
            Type::Custom(_id) => panic!("Custom types mapping not implemented yet"), 
        }
    }
}

pub struct Symbol {
    pub name: IdentifierId,
    pub symbol_type: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentifierId(pub u32);

pub struct SymbolTable {
    symbols: Vec<Symbol>,
    scope_markers: Vec<usize>, 
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scope_markers: vec![0], 
        }
    }

    pub fn enter_scope(&mut self) {
        self.scope_markers.push(self.symbols.len());
    }

    pub fn exit_scope(&mut self) {
        if self.scope_markers.len() > 1 {
            if let Some(previous_len) = self.scope_markers.pop() {
                self.symbols.truncate(previous_len);
            }
        }
    }

    pub fn insert(&mut self, symbol: Symbol) -> Result<(), SymbolTableError>{
        let current_scope_start = *self.scope_markers.last().unwrap();
        
        for existing in self.symbols[current_scope_start..].iter() {
            if existing.name == symbol.name {
                return Err(SymbolTableError::DuplicateSymbol { name: symbol.name });
            }
        }
        
        self.symbols.push(symbol);
        Ok(())
    }

    pub fn lookup(&self, name: IdentifierId) -> Result<&Symbol, SymbolTableError>  {
        for symbol in self.symbols.iter().rev() {
            if symbol.name == name {
                return Ok(symbol);
            }
        }
        Err(SymbolTableError::SymbolNotFound { name })
    }
}

pub struct Interner {
    map: HashMap<&'static str, IdentifierId>,
    storage: Vec<Box<str>>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            storage: Vec::new(),
        }
    }

    pub fn intern(&mut self, name: &str) -> IdentifierId {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        let permanent_box: Box<str> = name.into();
        let ptr: *const str = &*permanent_box;
    
        let static_ref: &'static str = unsafe { &*ptr };

        let id = IdentifierId(self.storage.len() as u32);

        self.storage.push(permanent_box);
        self.map.insert(static_ref, id.clone());

        id
    }

    pub fn lookup(&self, id: IdentifierId) -> Option<&str> {
        self.storage.get(id.0 as usize).map(|b| b.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolTableError {
    DuplicateSymbol { name: IdentifierId },
    SymbolNotFound { name: IdentifierId },
    ScopeUnderflow,
}
