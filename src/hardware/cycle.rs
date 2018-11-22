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
    ( $self:ident, $addr:expr ) => {{
        let data = $self.bus.read($addr);
        cycle!($self);
        data
    }}
}

// Read an address at PC and advances it.
macro_rules! cycle_fetch {
    ( $self:ident ) => {{
        let data = $self.bus.read($self.cpu.pc);
        $self.cpu.inc_pc();
        cycle!($self);
        data
    }}
}

// Read the next byte and discard it
macro_rules! cycle_implied { ( $self:ident ) => { cycle_read!($self, $self.cpu.pc); } }

// Fetch the next byte
macro_rules! cycle_immediate { ( $self:ident ) => { cycle_fetch!($self) } }

// Read an address at Zero Page.
macro_rules! cycle_zero_page {
    ( $self:ident ) => {{
        let addr = cycle_fetch!($self);
        cycle_read!($self, addr.into())
    }}
}

// Read an address at Zero Page indexed.
macro_rules! cycle_zero_page_indexed {
    ( $self:ident, $index:expr ) => {{
        let addr = cycle_fetch!($self);

        // Third cycle reads a value and adds the index to it, but won't use the result.
        cycle_read!($self, addr.into());

        cycle_read!($self, addr.wrapping_add($index).into())
    }}
}

// Read an address at Zero Page X.
macro_rules! cycle_zero_page_x {
    ( $self:ident ) => { cycle_zero_page_indexed!($self, $self.cpu.x) }
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

        let mut data = cycle_read!($self, (msb << 8) | (lsb as u16));

        // If overflow, msb needs to be adjusted
        if overflow { data = cycle_read!($self, (msb.wrapping_add(1) << 8) | (lsb as u16)); }

        data
    }}
}

// Read an address at Absolute X.
macro_rules! cycle_absolute_x {
    ( $self:ident ) => { cycle_absolute_indexed!($self, $self.cpu.x) }
}

// Read an address at Absolute Y.
macro_rules! cycle_absolute_y {
    ( $self:ident ) => { cycle_absolute_indexed!($self, $self.cpu.y) }
}
