use starknet::ContractAddress;

// Events with basic types
#[event]
#[derive(Drop, starknet::Event)]
enum BasicTypesEvents {
    U8Event: U8Event,
    U16Event: U16Event,
    U32Event: U32Event,
    U64Event: U64Event,
    U128Event: U128Event,
    U256Event: U256Event,
    Felt252Event: Felt252Event,
    BoolEvent: BoolEvent,
}

#[derive(Drop, starknet::Event)]
struct U8Event {
    #[key]
    value: u8,
}

#[derive(Drop, starknet::Event)]
struct U16Event {
    #[key]
    value: u16,
}

#[derive(Drop, starknet::Event)]
struct U32Event {
    #[key]
    value: u32,
}

#[derive(Drop, starknet::Event)]
struct U64Event {
    #[key]
    value: u64,
}

#[derive(Drop, starknet::Event)]
struct U128Event {
    #[key]
    value: u128,
}

#[derive(Drop, starknet::Event)]
struct U256Event {
    #[key]
    value: u256,
}

#[derive(Drop, starknet::Event)]
struct Felt252Event {
    #[key]
    value: felt252,
}

#[derive(Drop, starknet::Event)]
struct BoolEvent {
    #[key]
    value: bool,
}

// Events with complex types
#[event]
#[derive(Drop, starknet::Event)]
enum ComplexTypesEvents {
    OptionEvent: OptionEvent,
    AddressEvent: AddressEvent,
}

#[derive(Drop, starknet::Event)]
struct OptionEvent {
    #[key]
    optional_value: Option<felt252>,
}

#[derive(Drop, starknet::Event)]
struct AddressEvent {
    #[key]
    address: ContractAddress,
}

// Events with different structures
#[event]
#[derive(Drop, starknet::Event)]
enum StructureEvents {
    NoParamsEvent: NoParamsEvent,
    SingleParamEvent: SingleParamEvent,
    MultipleParamsEvent: MultipleParamsEvent,
    IndexedParamsEvent: IndexedParamsEvent,
    MixedParamsEvent: MixedParamsEvent,
}

#[derive(Drop, starknet::Event)]
struct NoParamsEvent {}

#[derive(Drop, starknet::Event)]
struct SingleParamEvent {
    #[key]
    value: felt252,
}

#[derive(Drop, starknet::Event)]
struct MultipleParamsEvent {
    #[key]
    param1: felt252,
    #[key]
    param2: u256,
    #[key]
    param3: bool,
}

#[derive(Drop, starknet::Event)]
struct IndexedParamsEvent {
    #[key]
    indexed_param: felt252,
    non_indexed_param: felt252,
}

#[derive(Drop, starknet::Event)]
struct MixedParamsEvent {
    #[key]
    indexed_felt: felt252,
    #[key]
    indexed_u256: u256,
    non_indexed_felt: felt252,
    non_indexed_bool: bool,
}

// Storage
#[starknet::interface]
trait IEventTestContract<TContractState> {
    fn emit_basic_types_events(ref self: TContractState);
    fn emit_complex_types_events(ref self: TContractState);
    fn emit_structure_events(ref self: TContractState);
    fn emit_all_events(ref self: TContractState);
}

#[starknet::contract]
mod EventTestContract {
    use super::IEventTestContract;
    use super::BasicTypesEvents;
    use super::ComplexTypesEvents;
    use super::StructureEvents;
    use super::U8Event;
    use super::U16Event;
    use super::U32Event;
    use super::U64Event;
    use super::U128Event;
    use super::U256Event;
    use super::Felt252Event;
    use super::BoolEvent;
    use super::OptionEvent;
    use super::AddressEvent;
    use super::NoParamsEvent;
    use super::SingleParamEvent;
    use super::MultipleParamsEvent;
    use super::IndexedParamsEvent;
    use super::MixedParamsEvent;

    #[storage]
    struct Storage {
        event_counter: u32,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        BasicTypesEvents: BasicTypesEvents,
        ComplexTypesEvents: ComplexTypesEvents,
        StructureEvents: StructureEvents,
    }

    #[abi(embed_v0)]
    impl EventTestContractImpl of IEventTestContract<ContractState> {
        fn emit_basic_types_events(ref self: ContractState) {
            // Emit events with basic types
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::U8Event(U8Event { value: 255 })));
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::U16Event(U16Event { value: 65535 })));
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::U32Event(U32Event { value: 4294967295 })));
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::U64Event(U64Event { value: 18446744073709551615 })));
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::U128Event(U128Event { value: 340282366920938463463374607431768211455 })));
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::U256Event(U256Event { value: 115792089237316195423570985008687907853269984665640564039457584007913129639935 })));
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::Felt252Event(Felt252Event { value: 'test_felt252' })));
            self.emit(Event::BasicTypesEvents(BasicTypesEvents::BoolEvent(BoolEvent { value: true })));
        }

        fn emit_complex_types_events(ref self: ContractState) {
            // Emit events with complex types
            self.emit(Event::ComplexTypesEvents(ComplexTypesEvents::OptionEvent(OptionEvent {
                optional_value: Option::Some('optional_value'),
            })));

            self.emit(Event::ComplexTypesEvents(ComplexTypesEvents::AddressEvent(AddressEvent {
                address: starknet::contract_address_const::<0x123456789012345678901234567890123456789012345678901234567890123>(),
            })));
        }

        fn emit_structure_events(ref self: ContractState) {
            // Emit events with different structures
            self.emit(Event::StructureEvents(StructureEvents::NoParamsEvent(NoParamsEvent {})));

            self.emit(Event::StructureEvents(StructureEvents::SingleParamEvent(SingleParamEvent {
                value: 'single_param',
            })));

            self.emit(Event::StructureEvents(StructureEvents::MultipleParamsEvent(MultipleParamsEvent {
                param1: 'param1',
                param2: 1000000000000000000,
                param3: true,
            })));

            self.emit(Event::StructureEvents(StructureEvents::IndexedParamsEvent(IndexedParamsEvent {
                indexed_param: 'indexed',
                non_indexed_param: 'non_indexed',
            })));

            self.emit(Event::StructureEvents(StructureEvents::MixedParamsEvent(MixedParamsEvent {
                indexed_felt: 'indexed_felt',
                indexed_u256: 2000000000000000000,
                non_indexed_felt: 'non_indexed_felt',
                non_indexed_bool: false,
            })));
        }

        fn emit_all_events(ref self: ContractState) {
            // Emit all types of events in sequence
            self.emit_basic_types_events();
            self.emit_complex_types_events();
            self.emit_structure_events();
        }
    }
}
