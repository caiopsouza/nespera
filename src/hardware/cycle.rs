// Pipe um result into another for chaining.
macro_rules! pipe {
    ( $initial:expr $( => $s:ident $( .$ident:ident )* )* ) => {{
        let res = $initial;
        $( let res = $s $( .$ident )* (res); )*
        res
    }}
}

// Helper to yield a cycle.
macro_rules! cycle {
    ( $self:ident ) => {{
        $self.cycle += 1;
        yield false;
    }}
}

// Read a value at the specified address.
macro_rules! cycle_read {
    // Read directly through the address
    ( $self:ident, $addr:expr ) => {{
        let data = $self.bus.read($addr);
        cycle!($self);
        data
    }};

    // Combine two bytes to make the address
    ( $self:ident, $lsb:expr, $msb:expr ) => {
        cycle_read!($self, ($lsb as u16) | (($msb as u16) << 8))
    }
}

// Write a value at the specified address.
macro_rules! cycle_write {
    // Read directly through the address
    ( $self:ident, $addr:expr, $data:expr ) => {{
        $self.bus.write($addr, $data);
        cycle!($self);
    }};

    // Combine two bytes to make the address
    ( $self:ident, $lsb:expr, $msb:expr, $data:expr ) => {{
        let data = $data;
        cycle_write!($self, ($lsb as u16) | (($msb as u16) << 8), data);
    }}
}

// Read an address and discard its value.
// Some read operations can have side effects, so they are necessary even if their value is not used.
macro_rules! cycle_dummy_read {
    ( $self:ident, $addr:expr ) => {{ cycle_read!($self, $addr); }}
}

// Read an address at PC and advances it.
macro_rules! cycle_fetch {
    ( $self:ident ) => {{
        let data = $self.bus.read($self.cpu.get_pc());
        $self.cpu.inc_pc();
        cycle!($self);
        data
    }}
}

// The implied argument is always read and discarded.
macro_rules! cycle_implied { ( $self:ident ) => { cycle_dummy_read!($self, $self.cpu.get_pc()); } }

// Fetch an immediate argument
macro_rules! cycle_immediate { ( $self:ident ) => { cycle_fetch!($self) } }

// Read an address at Zero Page.
macro_rules! cycle_zero_page {
    // Read
    ( $self:ident ) => {{
        let addr = cycle_fetch!($self);
        cycle_read!($self, addr.into())
    }};

    // Write
    ( $self:ident, $data:expr ) => {{
        let addr = cycle_fetch!($self);
        cycle_write!($self, addr.into(), $data);
    }}
}

// Read an address at Zero Page indexed.
macro_rules! cycle_zero_page_indexed {
    ( $self:ident, $index:expr ) => {{
        let addr = cycle_fetch!($self);
        cycle_dummy_read!($self, addr.into());
        cycle_read!($self, addr.wrapping_add($index).into())
    }}
}

// Read an address at Zero Page X.
macro_rules! cycle_zero_page_x {
    ( $self:ident ) => { cycle_zero_page_indexed!($self, $self.cpu.get_x()) }
}

// Read an address at Zero Page Y.
macro_rules! cycle_zero_page_y {
    ( $self:ident ) => { cycle_zero_page_indexed!($self, $self.cpu.get_y()) }
}

// Read an Absolute address.
macro_rules! cycle_absolute {
    ( $self:ident ) => {{
        let lsb: u16 = cycle_fetch!($self).into();
        let msb: u16 = cycle_fetch!($self).into();
        cycle_read!($self, (msb << 8) | lsb)
    }}
}

// Read an Absolute address indexed.
macro_rules! cycle_absolute_indexed {
    ( $self:ident, $index:expr ) => {{
        let lsb = cycle_fetch!($self);

        let msb: u16 = cycle_fetch!($self).into();
        let (lsb, overflow) = lsb.overflowing_add($index);

        let mut data = cycle_read!($self, lsb, msb);

        // If overflow, msb needs to be adjusted
        if overflow { data = cycle_read!($self, lsb, msb.wrapping_add(1)); }

        data
    }}
}

// Read an address at Absolute X.
macro_rules! cycle_absolute_x {
    ( $self:ident ) => { cycle_absolute_indexed!($self, $self.cpu.get_x()) }
}

// Read an address at Absolute Y.
macro_rules! cycle_absolute_y {
    ( $self:ident ) => { cycle_absolute_indexed!($self, $self.cpu.get_y()) }
}

// Indexed Indirect by X.
// If reading from 0x??FF, next byte will be from 0x??00 instead of 0x??00 + 0x0100.
macro_rules! cycle_indirect_x {
    ( $self:ident ) => {{
        let addr = cycle_fetch!($self);
        cycle_dummy_read!($self, addr.into());
        let lsb = cycle_read!($self, addr.wrapping_add($self.cpu.get_x()).into());
        let msb = cycle_read!($self, addr.wrapping_add($self.cpu.get_x()).wrapping_add(1).into());
        cycle_read!($self, lsb, msb)
    }}
}


// Indexed Indirect by Y.
macro_rules! cycle_indirect_y {
    ( $self:ident ) => {{
        let addr = cycle_fetch!($self);

        let lsb = cycle_read!($self, addr.into());
        let msb = cycle_read!($self, addr.wrapping_add(1).into());

        let (lsb, overflow) = lsb.overflowing_add($self.cpu.get_y());
        let mut data = cycle_read!($self, lsb, msb);

        // If overflow, msb needs to be adjusted
        if overflow { data = cycle_read!($self, lsb, msb.wrapping_add(1)); }

        data
    }}
}
