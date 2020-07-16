use crate::ST7789_wrapper::{
    self,
    SPIDriver
};
use crate::delay::TimerDelay;
use crate::hrs3300::I2cDriver;
use embedded_hal::{
    digital::v2::OutputPin
};
use nrf52832_hal::{
    self as hal,
    pac,
    gpio::{
        self,
        Level,
    },
    spim,
    twim
};
use crate::backlight::Backlight;
use crate::battery::BatteryStatus;

pub struct Components {
    pub display_wrapper: ST7789_wrapper::SPIDriver<
        gpio::p0::P0_26<gpio::Output<gpio::PushPull>>, 
        spim::Spim<pac::SPIM1>, 
        gpio::p0::P0_18<gpio::Output<gpio::PushPull>>, 
        TimerDelay<pac::TIMER0>>,    
    pub sensor: I2cDriver,
    pub backlight: Backlight,
    pub battery: BatteryStatus,
    pub delay_provider: TimerDelay<pac::TIMER1>,
}
impl Components {
    pub fn new() -> Components {
        let display_wrapper: ST7789_wrapper::SPIDriver<
            gpio::p0::P0_26<gpio::Output<gpio::PushPull>>, 
            spim::Spim<pac::SPIM1>, 
            gpio::p0::P0_18<gpio::Output<gpio::PushPull>>, 
            TimerDelay<pac::TIMER0>>;    
        let sensor: I2cDriver;
        let backlight: Backlight;
        let battery: BatteryStatus;
        let delay_provider: TimerDelay<pac::TIMER1>;

        let pac::Peripherals {
            CLOCK: clock_peripheral,
            // FICR,
            P0: p0_peripheral,
            // RADIO,
            SAADC: saadc_peripheral,
            // SPIM1,
            TIMER0: timer0_peripheral,
            TIMER1: timer1_peripheral,
            TWIM0: twim0_peripheral,
            SPIM1: spim1_peripheral,
            ..
        } = pac::Peripherals::take().unwrap();
    
        // Set up GPIO peripheral
        let gpio = gpio::p0::Parts::new(p0_peripheral);
    
        // Clock for bluetooth
        {
            // Set up clocks. On reset, the high frequency clock is already used,
            // but we also need to switch to the external HF oscillator. This is
            // needed for Bluetooth to work.
            let _clocks = hal::clocks::Clocks::new(clock_peripheral).enable_ext_hfosc();
        }
    
        // LCD
        {
            // Set up SPI pins
            let spi_clk = gpio.p0_02
                .into_push_pull_output(Level::Low).degrade();
            let spi_mosi = gpio.p0_03
                .into_push_pull_output(Level::Low).degrade();
            let spi_miso = gpio.p0_04
                .into_floating_input().degrade();
            let spi_pins = spim::Pins {
                sck: spi_clk,
                miso: Some(spi_miso),
                mosi: Some(spi_mosi)
            };
    
            // Set up LCD pins
            // LCD_RS - data/clock pin      (P0.18) 	Clock/data pin (CD)
            let lcd_data_clock = gpio.p0_18
                .into_push_pull_output(Level::Low);
            // LCD_CS - chip select         (P0.25) 	Chip select
            let mut lcd_chip_select = gpio.p0_25
                .into_push_pull_output(Level::Low);
            // LCD_RESET - reset            (P0.26) 	Display reset
            let lcd_reset = gpio.p0_26
                .into_push_pull_output(Level::Low);
    
            // Initialize SPI
            let spi_interface = spim::Spim::new(
                spim1_peripheral,
                spi_pins,
                // Use SPI at 8MHz (the fastest clock available on the nRF52832)
                // because otherwise refreshing will be super slow.
                spim::Frequency::M8,
                // SPI must be used in mode 3. Mode 0 (the default) won't work.
                spim::MODE_3,
                0);
    
            // Chip select must be held low while driving the display. It must be high
            // when using other SPI devices on the same bus (such as external flash
            // storage) so that the display controller won't respond to the wrong
            // commands.
            lcd_chip_select.set_low().unwrap();
    
            // Set up delay provider on TIMER0
            let delay_provider_0 = TimerDelay::new(timer0_peripheral);
            // Initialize LCD
            let display_driver = st7789::ST7789::new(
                    spi_interface,
                    lcd_data_clock, lcd_reset,
                    ST7789_wrapper::LCD_W, ST7789_wrapper::LCD_H,
                    delay_provider_0);
    
            display_wrapper = SPIDriver::new(display_driver);
        }
    
        // Sensor
        {
            // P0.06 : I²C SDA
            let sda = gpio.p0_06.into_floating_input().degrade();
            // P0.07 : I²C SCL
            let scl = gpio.p0_07.into_floating_input().degrade();
            // pins for TWIM0
            let pins = twim::Pins { scl, sda };    
            let twim_driver = twim::Twim::new(
                twim0_peripheral, 
                pins, 
                hal::target::twim0::frequency::FREQUENCY_A::K400
            );
            sensor = I2cDriver::new(twim_driver);
        }  
        
        // Backlight
        {
            backlight = Backlight::init(
                gpio.p0_14.into_push_pull_output(Level::High).degrade(),
                gpio.p0_22.into_push_pull_output(Level::High).degrade(),
                gpio.p0_23.into_push_pull_output(Level::High).degrade(),
                1,
            );
        }
    
        // Battery Status
        {
            battery = BatteryStatus::init(
                gpio.p0_12.into_floating_input(),
                gpio.p0_31.into_floating_input(),
                saadc_peripheral,
            ); 
        }
    
        // Delay provider
        {
            delay_provider = TimerDelay::new(timer1_peripheral);
        }

        Components {
            display_wrapper,
            sensor,
            backlight,
            battery,
            delay_provider
        }
    }
}