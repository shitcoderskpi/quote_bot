use steel::SteelVal;
use steel::gc::Gc;
use steel::rvals::Custom;

#[derive(Debug, Clone)]
pub struct SteelImage {
    data: Box<Vec<u8>>
}

impl SteelImage {
    pub fn new(data: Box<Vec<u8>>) -> Self {
        Self { data }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn into_steelval(self) -> SteelVal {
        SteelVal::Custom(Gc::new_mut(Box::new(self)))
    }

    pub fn from_steelval(steelval: SteelVal) -> Result<SteelImage, String> {
        let b = match steelval {
            SteelVal::Custom(b) => b,
            _ => return Err(format!("Type mismatch of steel val: {:?}", steelval)),
        };

        let guard = b.borrow();

        guard.as_any_ref()
            .downcast_ref::<SteelImage>()
            .cloned()
            .ok_or_else(|| {
                format!("Type mismatch: expected SteelImage, found: {}",
                        guard.name()
                )
            })
    }
}

impl Custom for SteelImage {}