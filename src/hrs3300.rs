use nrf52832_hal::{
    twim,
    pac,
};
use embedded_hal::blocking::i2c::WriteRead;

pub const SENSOR_ADDR: u8 = 0x44;
#[allow(unused)]
pub const DEVICE_ID: u8 = 0x21;
const SAMPLE_BLOCK_LEN: usize = 7;

pub type SensorError = twim::Error;
type SensorTwim = twim::Twim<pac::TWIM0>;

pub struct Sensor {
    i2c: SensorTwim,
    adc_wait_time_us: u32,
    resolution_mask: u32
}

impl Sensor {
    pub fn new(i2c: SensorTwim) -> Self {
        Sensor {
            i2c,
            adc_wait_time_us: 1250,
            resolution_mask: (1 << 15) - 1
        }
    }

    pub fn init(&mut self) -> Result<(), SensorError> {
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
    pub fn get_id(&mut self) -> Result<u8, SensorError> {
        self.reg_read(RegAddrs::ENABLE)
    }

    pub fn set_hrs_active(&mut self, active: bool) -> Result<(), SensorError> {
        let mut reg_data = self.reg_read(RegAddrs::ENABLE)?; 

        let value: u8 = active as u8;
        // bit 7 on/off 
        Self::write_bits(value, &mut reg_data, 7, 1);

        self.reg_write(RegAddrs::ENABLE, reg_data)
    }

    pub fn set_adc_wait_time(&mut self, wt: ADCWaitTime) -> Result<(), SensorError> {
        let mut reg_data = self.reg_read(RegAddrs::ENABLE)?;  

        self.adc_wait_time_us = wt.get_us();

        let value = wt as u8;
        // write to bits 4:6 of ENABLE
        Self::write_bits(value, &mut reg_data, 4, 3);

        self.reg_write(RegAddrs::ENABLE, reg_data)
    }

    pub fn get_adc_wait_time_us(&self) -> u32 {
        self.adc_wait_time_us
    }

    pub fn set_led_current(&mut self, lc: LedCurrent) -> Result<(), SensorError> {
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

    pub fn set_osc_active(&mut self, active: bool) -> Result<(), SensorError> {
        let mut reg_data = self.reg_read(RegAddrs::PDRIVER)?;  

        let value: u8 = active as u8;
        // // write to bit 5 of PDRIVER
        Self::write_bits(value, &mut reg_data, 5, 1);

        self.reg_write(RegAddrs::PDRIVER, reg_data)
    }

    pub fn set_gain(&mut self, gain: Gain) -> Result<(), SensorError> {
        let mut reg_data = self.reg_read(RegAddrs::HGAIN)?;  

        let value = gain as u8;
        // write to bit 4:2 of HGAIN
        Self::write_bits(value, &mut reg_data, 2, 3);

        self.reg_write(RegAddrs::HGAIN, reg_data)
    }

    pub fn set_resolution(&mut self, res: BitsResolution) -> Result<(), SensorError> {
        let mut reg_data = self.reg_read(RegAddrs::RES)?;  

        let value = res as u8;
        // write to bits 3:0 of RES
        Self::write_bits(value, &mut reg_data, 0, 4);

        self.resolution_mask = res.get_mask();

        self.reg_write(RegAddrs::RES, reg_data)
    }


    #[allow(non_snake_case)]
    pub fn read_raw_sample(&mut self) -> Result<RawSample, SensorError> {
        let mut sample_buff = [0u8; SAMPLE_BLOCK_LEN];
        self.read_registers(RegAddrs::C1DATAM, &mut sample_buff)?;

        // The order of returned data is:
        // 0: C1DATAM 0x08
        // 1: C0DATAM 0x09
        // 2: C0DATAH 0x0A
        // 3: PDRIVER
        // 4: C1DATAH 0x0D
        // 5: C1DATAL 0x0E
        // 6: C0DATAL 0x0F
        let ch1_0x08 = sample_buff[0]; 
        let ch0_0x09 = sample_buff[1]; 
        let ch0_0x0A = sample_buff[2];         
        let ch1_0x0D = sample_buff[4]; 
        let ch1_0x0E = sample_buff[5]; 
        let ch0_0x0F = sample_buff[6]; 
        
        let mut als: AlsValue = 0_u32;
        // 2:0 0x0E C1DATA[2:0] C1DATAL
        Self::extract_channel_bits(ch1_0x0E, 0, &mut als, 0, 3);        
        // 7:0 0x08 C1DATA[10:3] C1DATAM
        Self::extract_channel_bits(ch1_0x08, 0, &mut als, 3, 8);        
        // 6:0 0x0D C1DATA[17:11] C1DATAH
        Self::extract_channel_bits(ch1_0x0D, 0, &mut als, 11, 7);
        als &= self.resolution_mask;

        let mut hrs: HrsValue = 0_u32;
        // 3:0 0x0F C0DATA[3:0] C0DATAL
        Self::extract_channel_bits(ch0_0x0F, 0, &mut hrs, 0, 4);
        // 3:0 0x0A C0DATA[7:4] C0DATAH
        Self::extract_channel_bits(ch0_0x0A, 0, &mut hrs, 4, 4);
        // 7:0 0x09 C0DATA[15:8] C0DATAM
        Self::extract_channel_bits(ch0_0x09, 0, &mut hrs, 8, 8);
        // 5:4 0x0F C0DATA[17:16] C0DATAL
        Self::extract_channel_bits(ch0_0x0F, 4, &mut hrs, 16, 2);
        hrs &= self.resolution_mask;

        Ok(RawSample::new(hrs, als))
    }


    fn reg_write(&mut self, sensor_reg_addr: RegAddrs, value: u8) -> Result<(), SensorError> {
        let tr = [sensor_reg_addr as u8, value];

        self.i2c.write(SENSOR_ADDR, &tr).unwrap();

        Ok(())
    }

    fn reg_read(&mut self, sensor_reg_addr: RegAddrs) -> Result<u8, SensorError> {
        let mut buff = [0_u8; 1];
        let tr = [sensor_reg_addr as u8];

        self.i2c.write_read(SENSOR_ADDR, &tr, &mut buff).unwrap();

        Ok(buff[0])
    }

    fn read_registers(&mut self, start_register: RegAddrs, buffer_to: &mut [u8]) -> Result<(), SensorError> {
        let start_reg_bytes = [start_register as u8];
        self.i2c.write_read(SENSOR_ADDR, &start_reg_bytes, buffer_to).unwrap();

        Ok(())
    }


    fn write_bits(mut from_right_aligned: u8, to: &mut u8, start_to: usize, count: usize) {
        // assert!(start_to + count <= 8);
    
        let mask: u8 = (1 << count) - 1;   // create mask with ones in places 0..n, where n = 'start_from'
        from_right_aligned &= mask;        // extract bits from 'from'
        *to &= !(mask << start_to);        // clear bits in 'to'
    
        from_right_aligned <<= start_to;   // shift prepared 'from' to align it with 'to'
        *to |= from_right_aligned;         // put bits in 'to'
    }
    
    fn extract_channel_bits(from: u8, start_from: usize, to: &mut u32, start_to: usize, count: usize) {
        // assert!(start_from + count <= 8);
        // assert!(start_to + count <= 32);
    
        let mut from = from as u32;
    
        from >>= start_from;              // align 'from' value to the right
    
        let mask: u32 = (1 << count) - 1; // create mask with ones in places 0..n, where n = 'start_from'
        from &= mask;                     // extract bits from 'from'
        *to &= !(mask << start_to);        // clear bits in 'to'
    
        from <<= start_to;                // shift prepared 'from' to align it with 'to'
        *to |= from;                      // put bits in 'to'
    }
}

pub type HrsValue = u32;
pub type AlsValue = u32;
#[derive(Copy, Clone)]
pub struct RawSample {
    pub hrs: HrsValue,
    pub als: AlsValue
}
impl RawSample {
    pub fn new(hrs: HrsValue, als: AlsValue) -> Self {
        RawSample { hrs, als }
    }

    pub fn get_sum(&self) -> u32 {
        self.hrs.saturating_sub(self.als)
    }
}


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
impl ADCWaitTime {
    pub fn get_us(&self) -> u32 {
        match *self {
            ADCWaitTime::Ms800  => 800_000,
            ADCWaitTime::Ms400  => 400_000,
            ADCWaitTime::Ms200  => 200_000,
            ADCWaitTime::Ms100  => 100_000,
            ADCWaitTime::Ms75   => 75_000,
            ADCWaitTime::Ms50   => 50_000,
            ADCWaitTime::Ms12_5 => 1_250,
            ADCWaitTime::Ms0    => 0,
        }
    }
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
#[derive(Clone, Copy, Debug)]
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
impl BitsResolution {
    pub fn get_mask(&self) -> u32 {
        (1 << (*self as u8 + 8)) - 1
    }
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
