use nrf52832_hal::{ self as hal, twim, Twim };
use nrf52832_pac;

const SENSOR_ADDR: u8 = 0x44;
const BUFF_LEN: usize = 8;

pub struct PPG_Sensor {
    sensor: nrf52832_pac::Twim<nrf52832_pac::TWIM0>,
    read_buff: [u8; BUFF_LEN],
    write_buff: [u8; BUFF_LEN]
}

impl PPG_Sensor {
    pub fn new(gpio: hal::gpio::p0::Parts, 
                twim0: nrf52832_pac::TWIM0) -> PPG_Sensor {
        
        // up to 800 kHz
        // 525nm green
        trace!("sensor init starts");
        // P0.06 : I²C SDA
        let sda = gpio.p0_06.into_floating_input().degrade();
        // P0.07 : I²C SCL
        let scl = gpio.p0_07.into_floating_input().degrade();
        // pins for TWIM0
        let pins = twim::Pins { scl, sda };
        // sensor instance
        let mut sensor = PPG_Sensor {
            sensor: Twim::new(twim0, pins, 
                nrf52832_hal::target::twim0::frequency::FREQUENCY_A::K400),
            read_buff: [0_u8; BUFF_LEN],
            write_buff: [0_u8; BUFF_LEN]
        };

        //sensor setup


        trace!("sensor init ends"); 
        
        sensor
    }

    pub fn setup(&self) {
        match self.sensor.write_read(SENSOR_ADDR, &self.write_buff, &mut self.read_buff) {
            core::result::Result::Err(err) => {
                match err {
                    twim::Error::TxBufferTooLong => error!("TxBufferTooLong"),
                    twim::Error::RxBufferTooLong => error!("RxBufferTooLong"),
                    twim::Error::Transmit => error!("Transmit"),
                    twim::Error::Receive => error!("Receive"),
                    twim::Error::DMABufferNotInDataMemory => error!("DMABufferNotInDataMemory"),
                }
            },
            core::result::Result::Ok(_) => trace!("Sent ok")
        }
    }
}