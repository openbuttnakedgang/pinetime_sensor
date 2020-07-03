// use nrf52832_hal::{ self as hal, twim, Twim };
use embedded_hal::blocking::i2c;
use core::{ 
    result, 
    // marker, 
    // convert, 
    ops::Range 
};

const SENSOR_ADDR: u8 = 0x44;
const BUFF_LEN: usize = 8;
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
pub enum ADCWaitTime {
    Ms800,
    Ms400,
    Ms200,
    Ms100,
    Ms75,
    Ms50,
    Ms12_5,
    Ms0
}

// led current 2-bit value, bit 1 of 0:1
pub enum LedCurrent {
    Ma12_5,
    Ma20,
    Ma30,
    Ma40
}

// ADC resolution
pub enum BitsResolution {
    _8,
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
pub enum Gain {
    X1, 
    X2,
    X4,
    X8,
    X64
}

pub struct I2cDriver<I2C> {
    i2c: I2C
}

impl <I2C, E> I2cDriver<I2C> 
where 
    I2C: i2c::Read<Error = E> + i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug
{
    pub fn new(i2c: I2C) -> Self {
        I2cDriver {
            i2c
        }
    }

    fn reg_write(i2c: &mut I2C, sensor_reg_addr: RegAddrs, value: u8) -> result::Result<(), E> {
        let mut buff = [0_u8, 1];
        let tr = [sensor_reg_addr as u8, value];

        i2c.write(SENSOR_ADDR, &tr)?;
        i2c.write_read(SENSOR_ADDR, &tr[..1], &mut buff)?;  

        println!("reg {:X} val {:X}", tr[0], buff[0]);
        assert_eq!(tr[1], buff[0]);

        result::Result::Ok(())
    }
    fn reg_read(i2c: &mut I2C, sensor_reg_addr: RegAddrs) -> result::Result<u8, E> {
        let mut buff = [0_u8; 1];
        let tr = [sensor_reg_addr as u8];

        i2c.write_read(SENSOR_ADDR, &tr, &mut buff)?;

        result::Result::Ok(buff[0])
    }

    pub fn get_id(&mut self) -> result::Result<u8, E> {
        Self::reg_read(&mut self.i2c, RegAddrs::ENABLE)
    }

    pub fn set_hrs_active(&mut self, active: bool) -> result::Result<(), E> {
        let mut reg_data = Self::reg_read(&mut self.i2c, RegAddrs::ENABLE)?; 

        // bit 7 on/off 
        if active { 
            reg_data |= 1 << 7; 
        } else { 
            reg_data &= !(1 << 7); 
        }

        Self::reg_write(&mut self.i2c, RegAddrs::ENABLE, reg_data)
    }

    pub fn set_adc_wait_time(&mut self, wt: ADCWaitTime) -> result::Result<(), E> {
        let mut reg_data = Self::reg_read(&mut self.i2c, RegAddrs::ENABLE)?;  

        let value = match wt {
            ADCWaitTime::Ms800  => 0 << 4,
            ADCWaitTime::Ms400  => 1 << 4,
            ADCWaitTime::Ms200  => 2 << 4,
            ADCWaitTime::Ms100  => 3 << 4,
            ADCWaitTime::Ms75   => 4 << 4,
            ADCWaitTime::Ms50   => 5 << 4,
            ADCWaitTime::Ms12_5 => 6 << 4,
            ADCWaitTime::Ms0    => 7 << 4
        };
        // write to bits 4:6 of ENABLE
        Self::set_bits(&mut reg_data, 4..6, value);

        Self::reg_write(&mut self.i2c, RegAddrs::ENABLE, reg_data)
    }

    pub fn set_led_current(&mut self, lc: LedCurrent) -> result::Result<(), E> {
        let value = match lc {
            LedCurrent::Ma12_5  => 0, 
            LedCurrent::Ma20    => 1, 
            LedCurrent::Ma30    => 2,
            LedCurrent::Ma40    => 3  
        };

        let mut enable_data = Self::reg_read(&mut self.i2c, RegAddrs::ENABLE)?; 
        // extract bit 1 from lc and move it to 3
        let to_enable: u8 = (value & 0b10) << 2;
        // write to bit 3 of ENABLE
        Self::set_bits(&mut enable_data, 3..3, to_enable);
        Self::reg_write(&mut self.i2c, RegAddrs::ENABLE, enable_data)?;

        let mut pdriver_data = Self::reg_read(&mut self.i2c, RegAddrs::PDRIVER)?; 
        // extract bit 0 from lc and move it to 6
        let to_pdriver: u8 = (value & 0b01) << 6;
        // write to bit 6 of PDRIVER
        Self::set_bits(&mut pdriver_data, 6..6, to_pdriver);        
        Self::reg_write(&mut self.i2c, RegAddrs::PDRIVER, pdriver_data)
    }

    pub fn set_osc_active(&mut self, active: bool) -> result::Result<(), E> {
        let mut reg_data = Self::reg_read(&mut self.i2c, RegAddrs::PDRIVER)?;  

        // convert from bool to 5th bit
        let value: u8 = if active {
            1 << 5
        } else {
            0 << 5
        };
        // write to bit 5 of PDRIVER
        Self::set_bits(&mut reg_data, 5..5, value);

        Self::reg_write(&mut self.i2c, RegAddrs::PDRIVER, reg_data)
    }

    pub fn set_gain(&mut self, gain: Gain) -> result::Result<(), E> {
        let mut reg_data = Self::reg_read(&mut self.i2c, RegAddrs::HGAIN)?;  

        let value = match gain {
            Gain::X1  => 0 << 2, 
            Gain::X2  => 1 << 2, 
            Gain::X4  => 2 << 2, 
            Gain::X8  => 3 << 2, 
            Gain::X64 => 4 << 2, 
        };
        // write to bit 4:2 of HGAIN
        Self::set_bits(&mut reg_data, 2..4, value);

        Self::reg_write(&mut self.i2c, RegAddrs::HGAIN, reg_data)
    }

    pub fn set_resolution(&mut self, res: BitsResolution) -> result::Result<(), E> {
        let mut reg_data = Self::reg_read(&mut self.i2c, RegAddrs::RES)?;  

        let value = match res {
            BitsResolution::_8  => 0,
            BitsResolution::_9  => 1,
            BitsResolution::_10 => 2,
            BitsResolution::_11 => 3,
            BitsResolution::_12 => 4,
            BitsResolution::_13 => 5,
            BitsResolution::_14 => 6,
            BitsResolution::_15 => 7,
            BitsResolution::_16 => 8,
            BitsResolution::_17 => 9,
            BitsResolution::_18 => 10
        };
        // write to bits 3:0 of RES
        Self::set_bits(&mut reg_data, 0..3, value);

        Self::reg_write(&mut self.i2c, RegAddrs::RES, reg_data)
    }

    #[allow(non_snake_case)]
    pub fn get_ch0(&mut self) -> result::Result<u32, E> {
        let ch0_0x0F = Self::reg_read(&mut self.i2c, RegAddrs::C0DATAL)? as u8; 
        let ch0_0x09 = Self::reg_read(&mut self.i2c, RegAddrs::C0DATAM)? as u8; 
        let ch0_0x0A = Self::reg_read(&mut self.i2c, RegAddrs::C0DATAH)? as u8; 
        
        // 3:0 0x0F C0DATA[3:0] C0DATAL
        let bits0_3: u32    = (ch0_0x0F & 0b0000_0111) as u32; 
        // 3:0 0x0A C0DATA[7:4] C0DATAH
        let bits4_7: u32    = (ch0_0x0A & 0b0000_0111) as u32; 
        // 7:0 0x09 C0DATA[15:8] C0DATAM
        let bits8_15: u32   = (ch0_0x09 & 0b1111_1111) as u32; 
        // 5:4 0x0F C0DATA[17:16] C0DATAL
        let bits16_17: u32  = (ch0_0x0F & 0b0011_0000) as u32; 

        let value = bits0_3 | (bits4_7 << 4) | (bits8_15 << 8) | (bits16_17 << 16);

        Ok(value)
    }

    #[allow(non_snake_case)]
    pub fn get_ch1(&mut self) -> result::Result<u32, E> {
        let ch0_0x0E = Self::reg_read(&mut self.i2c, RegAddrs::C1DATAL)? as u8; 
        let ch0_0x08 = Self::reg_read(&mut self.i2c, RegAddrs::C1DATAM)? as u8; 
        let ch0_0x0D = Self::reg_read(&mut self.i2c, RegAddrs::C1DATAH)? as u8; 
        
        // 2:0 0x0E C1DATA[2:0] C1DATAL
        let bits0_2: u32    = (ch0_0x0E & 0b0000_0011) as u32; 
        // 7:0 0x08 C1DATA[10:3] C1DATAM
        let bits3_10: u32    = (ch0_0x08 & 0b1111_1111) as u32; 
        // 6:0 0x0D C1DATA[17:11] C1DATAH
        let bits11_17: u32   = (ch0_0x0D & 0b0111_1111) as u32; 

        let value = bits0_2 | (bits3_10 << 4) | (bits11_17 << 8);

        Ok(value)
    }

    fn set_bits(reg_data: &mut u8, bit_nums: Range<u8>, bits_in_byte: u8) {
        // create bit mask
        let mut mask = 0_u8;
        for bit_num in bit_nums {
            mask |= 1 << bit_num;
        }
        // clear bits by setting them to 0
        *reg_data &= !mask;
        // apply masked value
        *reg_data |= bits_in_byte & mask;
    }

    pub fn init(&mut self) -> result::Result<(), E> {
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
}


pub fn try_hrs3300<T, E> (sensor: &mut I2cDriver<T>) -> result::Result<(), E> 
where
    T:  i2c::Write::<Error = E> + 
        i2c::Read::<Error = E> + 
        i2c::WriteRead::<Error = E>,
    E:  core::fmt::Debug
{    
    info!("HRS3300 usage starts");

    info!("HRS3300 init:");
    sensor.init()?;
    info!("HRS3300 init successful!");

    info!("HRS3300 hrs activation:");
    sensor.set_hrs_active(true)?;
    info!("HRS3300 hrs activation successful!");

    info!("HRS3300 osc activation:");
    sensor.set_osc_active(true)?;
    info!("HRS3300 osc activation successful!");

    let count = 10;
    for _ in 0..count {
        info!("HRS3300 measure ch0 sample:");
        let value = sensor.get_ch0()?;
        info!("HRS3300 ch0 sample {}", value);

        info!("HRS3300 measure ch1 sample:");
        let value = sensor.get_ch1()?;
        info!("HRS3300 ch1 sample {}", value);
    }

    result::Result::<(), E>::Ok(())
}