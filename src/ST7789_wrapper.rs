use embedded_hal::{
    digital::v2::OutputPin,
    blocking::delay::DelayUs,
    blocking::spi
};
use core::result::Result;
use embedded_graphics::{
    style,
    prelude::*,
    pixelcolor,
    primitives::rectangle,
    fonts
};

const BACKGROUND_COLOR: pixelcolor::Rgb565 = pixelcolor::Rgb565::new(0, 0b000111, 0x0F);
const MARGIN: u16 = 10;
pub const LCD_W: u16 = 240;
pub const LCD_H: u16 = 240;

pub struct SPIDriver<RST, SPI, DC, DELAY> 
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    display_driver: st7789::ST7789::<SPI, DC, RST, DELAY>
}

impl<RST, SPI, DC, DELAY, E> SPIDriver<RST, SPI, DC, DELAY> 
where
    RST: OutputPin<Error = E>, 
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
    E: core::fmt::Debug
{
    pub fn new(display_driver: st7789::ST7789<SPI, DC, RST, DELAY>) -> Self {
        SPIDriver::<RST, SPI, DC, DELAY> { display_driver }
    }
    
    pub fn init(&mut self) -> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>> {
        self.display_driver.init()?;       
        self.display_driver.set_orientation(&st7789::Orientation::Portrait)?;
        Ok(())
    }

    pub fn draw_text(&mut self) -> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>>
            // <SPI as embedded_hal::blocking::spi::Write<u8>>::Error, 
            // <DC as embedded_hal::digital::v2::OutputPin>::Error, E>: core::fmt::Debug>
    {
        // Draw something onto the LCD
        let backdrop_style = style::PrimitiveStyleBuilder::new()
            .fill_color(BACKGROUND_COLOR)
            .build();

        rectangle::Rectangle::new(
            Point::new(0, 0), 
            Point::new(LCD_W as i32, LCD_H as i32)
        )
            .into_styled(backdrop_style)
            .draw(&mut self.display_driver)?;

        // Choose text style
        let text_style = style::TextStyleBuilder::new(fonts::Font12x16)
            .text_color(pixelcolor::Rgb565::WHITE)
            .background_color(BACKGROUND_COLOR);

        // Draw text
        fonts::Text::new("HRS data ...", Point::new(10, 10))
            .into_styled(text_style.build())
            .draw(&mut self.display_driver)?;

        // Draw text
        fonts::Text::new("20%", Point::new(10, 10 + 16 + MARGIN as i32))
            .into_styled(text_style.build())
            .draw(&mut self.display_driver)?;

        Ok(())
    }
}