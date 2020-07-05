// Display module, ST7789

// LCD_SCK                      (P0.02) 	SPI clock
// LCD_SDI                      (P0.03) 	SPI MOSI
// LCD_RS                       (P0.18) 	Clock/data pin (CD)
// LCD_CS                       (P0.25) 	Chip select
// LCD_RESET                    (P0.26) 	Display reset
// LCD_BACKLIGHT_{LOW,MID,HIGH}  Backlight (active low) 

use embedded_hal::blocking::spi;

pub struct SPI_Driver <SPI> 
{
    spi: SPI
}

impl<SPI, E> SPI_Driver<SPI>
where 
    SPI: spi::Write<u8, Error = E> + spi::Transfer<u8, Error = E>,
    E: core::fmt::Debug
{
    pub fn new(spi: SPI) -> SPI_Driver<SPI> {
        SPI_Driver {
            spi
        }
    }

    pub fn init() -> Result<(), E> {
        Ok(())
    }
}