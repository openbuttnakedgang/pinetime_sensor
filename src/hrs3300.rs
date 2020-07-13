// use nrf52832_hal::{ self as hal, twim, Twim };
use embedded_hal::blocking::i2c;
use core;

const SENSOR_ADDR: u8 = 0x44;
#[allow(unused)]
pub enum RegAddrs {
    ID = 0x00,      // R/W Device ID 0x21
    ENABLE = 0x01,  // R/W Enable HRS 0x68
    C1DATAM = 0x08, // RO CH1 data register bit 10~3 0x00
    C0DATAM = 0x09, // RO CH0 data register bit 15~8 0x00
    C0DATAH = 0x0A, // RO CH0 data register bit 7~4 0x00
    PDRIVER = 0x0C, // R/W HRS LED driver/PON/PDRIVE[0] 0x68
    C1DATAH = 0x0D, // RO CH1 data register bit 17~11 0x00
    C1DATAL = 0x0E, // RO CH1 data register bit 2~0 0x00
    C0DATAL = 0x0F, // RO CH1 data register bit 17~16 and 3~0 0x00
    RES = 0x16,     // R/W ALS and HRS resolution 0x66
    HGAIN = 0x17    // R/W HRS gain 0x10
}

// bits 4:6, wait time between each conversion 
#[allow(unused)]
pub enum ADCWaitTime {
    Ms800 = 0,
    Ms400,
    Ms200,
    Ms100,
    Ms75,
    Ms50,
    Ms12_5,
    Ms0
}

// led current 2-bit value, bit 1 of 0:1
#[allow(unused)]
pub enum LedCurrent {
    Ma12_5 = 0,
    Ma20,
    Ma30,
    Ma40
}

// ADC resolution
pub enum BitsResolution {
    _8 = 0,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
    _16,
    _17,
    _18
}

// gain
#[allow(unused)]
pub enum Gain {
    X1 = 0, 
    X2,
    X4,
    X8,
    X64
}

pub struct I2cDriver<I2C> {
    i2c: I2C,
    adc_wait_time_us : u32
}

impl<I2C, E> I2cDriver<I2C> 
where 
    I2C: i2c::Read<Error = E> + i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug
{
    pub fn new(i2c: I2C) -> Self {
        I2cDriver {
            i2c,
            adc_wait_time_us: 1250
        }
    }

    pub fn init(&mut self) -> Result<(), E> {
        // recommended values

        // ENABLE = 0x68 => 
        //   wait time = 12.5ms, .110_....
        //   led current = 40mA, ...._1... bit 1
        self.set_adc_wait_time(ADCWaitTime::Ms12_5)?;

        // PDRIVER = 0x68 => 
        //   led current = 40mA, .1.._.... bit 0
        //   osc = active,       ..1._....
        self.set_led_current(LedCurrent::Ma40)?;

        // RES = 0x66 => 
        //   reolution = 14 bits, ...._0110
        self.set_resolution(BitsResolution::_14)?;

        // HGAIN = 0x10 => 
        //   gain = x64,          ...1_00..
        self.set_gain(Gain::X64)?;

        Ok(())
    }
    
    #[allow(unused)]
    pub fn get_id(&mut self) -> Result<u8, E> {
        self.reg_read(RegAddrs::ENABLE)
    }

    pub fn set_hrs_active(&mut self, active: bool) -> Result<(), E> {
        let mut reg_data = self.reg_read(RegAddrs::ENABLE)?; 

        let value: u8 = active as u8;
        // bit 7 on/off 
        Self::write_bits(value, &mut reg_data, 7, 1);

        self.reg_write(RegAddrs::ENABLE, reg_data)
    }

    pub fn set_adc_wait_time(&mut self, wt: ADCWaitTime) -> Result<(), E> {
        let mut reg_data = self.reg_read(RegAddrs::ENABLE)?;  

        self.adc_wait_time_us = match wt {
            ADCWaitTime::Ms800  => 800_000,
            ADCWaitTime::Ms400  => 400_000,
            ADCWaitTime::Ms200  => 200_000,
            ADCWaitTime::Ms100  => 100_000,
            ADCWaitTime::Ms75   => 75_000,
            ADCWaitTime::Ms50   => 50_000,
            ADCWaitTime::Ms12_5 => 1_250,
            ADCWaitTime::Ms0    => 0,
        };

        let value = wt as u8;
        // write to bits 4:6 of ENABLE
        Self::write_bits(value, &mut reg_data, 4, 3);

        self.reg_write(RegAddrs::ENABLE, reg_data)
    }

    pub fn get_adc_wait_time_us(&self) -> u32 {
        self.adc_wait_time_us
    }

    pub fn set_led_current(&mut self, lc: LedCurrent) -> Result<(), E> {
        let value = lc as u8;

        let mut enable_data = self.reg_read( RegAddrs::ENABLE)?; 
        // extract bit 1 from lc and move it to 3
        let to_enable: u8 = value & 0b10;
        // write to bit 3 of ENABLE
        Self::write_bits(to_enable, &mut enable_data, 3, 1);
        self.reg_write(RegAddrs::ENABLE, enable_data)?;

        let mut pdriver_data = self.reg_read( RegAddrs::PDRIVER)?; 
        // extract bit 0 from lc and move it to 6
        let to_pdriver: u8 = (value & 0b01) << 6;
        // write to bit 6 of PDRIVER
        Self::write_bits(to_pdriver, &mut pdriver_data, 6, 1);        
        self.reg_write( RegAddrs::PDRIVER, pdriver_data)
    }

    pub fn set_osc_active(&mut self, active: bool) -> Result<(), E> {
        let mut reg_data = self.reg_read(RegAddrs::PDRIVER)?;  

        let value: u8 = active as u8;
        // // write to bit 5 of PDRIVER
        Self::write_bits(value, &mut reg_data, 5, 1);

        self.reg_write(RegAddrs::PDRIVER, reg_data)
    }

    pub fn set_gain(&mut self, gain: Gain) -> Result<(), E> {
        let mut reg_data = self.reg_read(RegAddrs::HGAIN)?;  

        let value = gain as u8;
        // write to bit 4:2 of HGAIN
        Self::write_bits(value, &mut reg_data, 2, 3);

        self.reg_write(RegAddrs::HGAIN, reg_data)
    }

    pub fn set_resolution(&mut self, res: BitsResolution) -> Result<(), E> {
        let mut reg_data = self.reg_read(RegAddrs::RES)?;  

        let value = res as u8;
        // write to bits 3:0 of RES
        println!(">: {:0>8b}", reg_data);
        Self::write_bits(value, &mut reg_data, 0, 4);
        println!("<: {:0>8b}", reg_data);

        self.reg_write(RegAddrs::RES, reg_data)
    }

    #[allow(non_snake_case)]
    pub fn get_ch0_hrs(&mut self) -> Result<u32, E> {
        let ch0_0x0F = self.reg_read(RegAddrs::C0DATAL)? as u8; 
        let ch0_0x09 = self.reg_read(RegAddrs::C0DATAM)? as u8; 
        let ch0_0x0A = self.reg_read(RegAddrs::C0DATAH)? as u8; 
        
        let mut value: u32 = 0_u32;

        // println!("0: {:0>8b}|{:0>8b}|{:0>8b}|{:0>8b}", ch0_0x0F, ch0_0x09, ch0_0x0A, ch0_0x0F);

        // 3:0 0x0F C0DATA[3:0] C0DATAL
        Self::extract_channel_bits(ch0_0x0F, 0, &mut value, 0, 4);

        // 3:0 0x0A C0DATA[7:4] C0DATAH
        Self::extract_channel_bits(ch0_0x0A, 0, &mut value, 4, 4);

        // 7:0 0x09 C0DATA[15:8] C0DATAM
        Self::extract_channel_bits(ch0_0x09, 0, &mut value, 8, 8);

        // 5:4 0x0F C0DATA[17:16] C0DATAL
        Self::extract_channel_bits(ch0_0x0F, 4, &mut value, 16, 2);

        Ok(value)
    }

    #[allow(non_snake_case)]
    pub fn get_ch1_als(&mut self) -> Result<u32, E> {
        let ch1_0x0E = self.reg_read(RegAddrs::C1DATAL)? as u8; 
        let ch1_0x08 = self.reg_read(RegAddrs::C1DATAM)? as u8; 
        let ch1_0x0D = self.reg_read(RegAddrs::C1DATAH)? as u8; 
        
        let mut value: u32 = 0_u32;

        // println!("1: {:0>8b}|{:0>8b}|{:0>8b}", ch1_0x0D & 0b0111_1111, ch1_0x08, ch1_0x0E & 0b0000_0111);

        // 2:0 0x0E C1DATA[2:0] C1DATAL
        Self::extract_channel_bits(ch1_0x0E, 0, &mut value, 0, 3);
        
        // 7:0 0x08 C1DATA[10:3] C1DATAM
        Self::extract_channel_bits(ch1_0x08, 0, &mut value, 3, 8);
        
        // 6:0 0x0D C1DATA[17:11] C1DATAH
        Self::extract_channel_bits(ch1_0x0D, 0, &mut value, 11, 7);

        Ok(value)
    }

    fn reg_write(&mut self, sensor_reg_addr: RegAddrs, value: u8) -> Result<(), E> {
        let tr = [sensor_reg_addr as u8, value];

        self.i2c.write(SENSOR_ADDR, &tr)?;

        Ok(())
    }

    fn reg_read(&mut self, sensor_reg_addr: RegAddrs) -> Result<u8, E> {
        let mut buff = [0_u8; 1];
        let tr = [sensor_reg_addr as u8];

        self.i2c.write_read(SENSOR_ADDR, &tr, &mut buff)?;

        Ok(buff[0])
    }

    fn write_bits(mut from_right_aligned: u8, to: &mut u8, start_to: usize, count: usize) {
        assert!(start_to + count <= 8);
    
        let mask: u8 = (1 << count) - 1;   // create mask with ones in places 0..n, where n = 'start_from'
        from_right_aligned &= mask;        // extract bits from 'from'
        *to &= !(mask << start_to);        // clear bits in 'to'
    
        from_right_aligned <<= start_to;   // shift prepared 'from' to align it with 'to'
        *to |= from_right_aligned;         // put bits in 'to'
    }
    
    fn extract_channel_bits(from: u8, start_from: usize, to: &mut u32, start_to: usize, count: usize) {
        assert!(start_from + count <= 8);
        assert!(start_to + count <= 32);
    
        let mut from = from as u32;
    
        from >>= start_from;              // align 'from' value to the right
    
        let mask: u32 = (1 << count) - 1; // create mask with ones in places 0..n, where n = 'start_from'
        from &= mask;                     // extract bits from 'from'
        *to &= !(mask << start_to);        // clear bits in 'to'
    
        from <<= start_to;                // shift prepared 'from' to align it with 'to'
        *to |= from;                      // put bits in 'to'
    }
}
