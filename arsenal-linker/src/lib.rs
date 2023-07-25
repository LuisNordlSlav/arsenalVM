use arsenal_globals::ArsenalObject;
use bincode::{deserialize, serialize};

pub fn link(obj: &mut ArsenalObject) -> &mut Vec<u8> {
    match obj {
        ArsenalObject::ArsenalCompiledObject { ref mut data } => data,
        ArsenalObject::ArsenalLibraryObject {} => todo!(),
    }
}

pub fn decode(data: Vec<u8>) -> ArsenalObject {
    deserialize::<ArsenalObject>(&data[..]).expect("invalid encoded object")
}

pub fn encode(data: &ArsenalObject) -> Vec<u8> {
    serialize::<ArsenalObject>(data).expect("failed to serialize data")
}
