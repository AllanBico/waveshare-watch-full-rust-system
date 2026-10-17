# Complete System Pin-to-Pin Netlist & Connection Breakdown

This document provides a structured, text-based graph representation of all components, interconnects, and functional blocks in the schematic set.

---

## 1. USB Interface Subsystem

* **Primary Components:** `U9` (USB Connector / Controller), `TVS1` (`LTVS16H5.0ET5G`)

### Interconnections:
* **`VBUS` Net:** 
  * Connected to `U9` pins `A4`, `B4`, `A9`, `B9`.
  * Connected to `TVS1` (Pin 1) for transient voltage suppression.
* **`GND` Net:**
  * Connected to `U9` shield ground pins (`A1`, `B1`, `A12`, `B12`) and exposed pad `0`.
  * Connected to `TVS1` (Pin 2).
* **Configuration Channel (`CC`) Pins:**
  * `U9` Pin `A5` (`CC1`) $\rightarrow$ `R2` ($5.1\text{ k}\Omega$) $\rightarrow$ `GND`.
  * `U9` Pin `B5` (`CC2`) $\rightarrow$ `R1` ($5.1\text{ k}\Omega$) $\rightarrow$ `GND`.
* **Differential Data Lines (`USB_N` / `USB_P`):**
  * `U9` Pin `A7` / `B7` (`DN1` / `DN2`) $\rightarrow$ Net `USB_N`.
  * `U9` Pin `A6` / `B6` (`DP1` / `DP2`) $\rightarrow$ Net `USB_P`.

---

## 2. Motor Driver Circuit

* **Primary Components:** `Q1` (`MMBT3904` NPN Transistor), `R7`, `R12`, `R13`, Connector Pads `P1`, `P2`

### Interconnections:
* **Power Supply Path:**
  * Rail `ALDO3` $\rightarrow$ Resistor `R7` ($0\Omega\text{ 0402}$) $\rightarrow$ Pad `P1`.
* **Base Control Logic:**
  * Net `GPIO18_MOTOR` $\rightarrow$ Series Resistor `R12` ($4.7\text{ k}\Omega$) $\rightarrow$ Base (`Q1` Pin 1).
  * Node between `R12` and Base $\rightarrow$ Pull-down Resistor `R13` ($47\text{ k}\Omega$) $\rightarrow$ `GND`.
* **Transistor Switching Loop:**
  * Emitter (`Q1` Pin 2) $\rightarrow$ `GND`.
  * Collector (`Q1` Pin 3) $\rightarrow$ Pad `P2`.

---

## 3. ESP32-S3 Core & Flash Subsystem

* **Primary Components:** `U2` (`ESP32-S3R8`), `U3` (`GD25Q256EYIGR` Flash), `X1` ($40\text{ MHz}$ Crystal)

### Main MCU (`U2`) Pin-to-Net Mapping:
* **Power Nets:**
  * Pins `2` (`VDD3P3`), `3` (`VDD3P3`), `56` (`VCC3V3`), `46` (`VCC3V3`) $\rightarrow$ Rail `VCC3V3`.
  * Decoupling: `C1` ($1\,\mu\text{F}$), `C2` ($100\text{ nF}$), `C7` ($100\text{ nF}$), `C13` ($100\text{ nF}$) connected to `GND`.
* **USB Interface:**
  * Pin `25` (`USB_N`) $\rightarrow$ `R19` ($22\Omega$) $\rightarrow$ Net `USB'_N` $\rightarrow$ `USB_N`.
  * Pin `26` (`USB_P`) $\rightarrow$ `R20` ($22\Omega$) $\rightarrow$ Net `USB'_P` $\rightarrow$ `USB_P`.
* **Crystal Oscillator (`X1`):**
  * Pin `54` (`XTAL_P`) $\rightarrow$ `X1` Pin 1 / Inductor `IND2` ($24\text{ nH}$) / Capacitor `C10` ($15\text{ pF}$) to `GND`.
  * Pin `53` (`XTAL_N`) $\rightarrow$ `X1` Pin 2 / Inductor `IND2` / Capacitor `C12` ($15\text{ pF}$) to `GND`.

### SPI Flash (`U3`) Pin Mapping:
* Pin 1 (`CS#`) $\rightarrow$ Net `SPICS0` $\rightarrow$ `U2` Pin 32 (`SPICS0`).
* Pin 2 (`SO/IO1`) $\rightarrow$ Net `SPIQ` $\rightarrow$ `U2` Pin 34 (`SPIQ`).
* Pin 3 (`WP#/IO2`) $\rightarrow$ Net `SPIWP` $\rightarrow$ `U2` Pin 31 (`SPIWP`).
* Pin 4 (`VSS`) $\rightarrow$ `GND`.
* Pin 5 (`SI/IO0`) $\rightarrow$ Net `SPID` $\rightarrow$ `U2` Pin 35 (`SPID`).
* Pin 6 (`SCLK`) $\rightarrow$ Net `SPICLK` $\rightarrow$ `U2` Pin 33 (`SPICLK`).
* Pin 7 (`HOLD#/IO3`) $\rightarrow$ Net `SPIHD` $\rightarrow$ `U2` Pin 30 (`SPIHD`).
* Pin 8 (`VCC`) $\rightarrow$ Rail `VDD_SPI` (Decoupled via `C4` $100\text{ nF}$, `C5` $100\text{ nF}$ to `GND`).

---

## 4. AMOLED Display Subsystem

* **Primary Components:** `J3` (Display Connector), Pull-Up Resistors `R26`, `R27`, `R28`

### Pin-to-Net Mapping:
* **Power Supply:**
  * Pins `23` (`VCI`), `25` (`VDDIO`) $\rightarrow$ Rail `VCC3V3`.
  * Decoupling: `C24` ($100\text{ nF}$), `C25` ($10\,\mu\text{F}$), `C27` ($10\,\mu\text{F}$), `C28` ($100\text{ nF}$) to `GND`.
* **QSPI Display Data Bus:**
  * Pin `4` (`QSPI_SCL`) $\rightarrow$ Net `QSPI_SCL` $\rightarrow$ `GPIO11`
  * Pin `6` (`LCD_CS`) $\rightarrow$ Net `LCD_CS` $\rightarrow$ `GPIO12`
  * Pin `8` (`QSPI_SI3`) $\rightarrow$ Net `QSPI_SI3` $\rightarrow$ `GPIO7`
  * Pin `10` (`QSPI_SI2`) $\rightarrow$ Net `QSPI_SI2` $\rightarrow$ `GPIO6`
  * Pin `12` (`QSPI_SI1`) $\rightarrow$ Net `QSPI_SI1` $\rightarrow$ `GPIO5`
  * Pin `14` (`QSPI_SI00`) $\rightarrow$ Net `QSPI_SI00` $\rightarrow$ `GPIO4`
  * Pin `19` (`LCD_RESET`) $\rightarrow$ Net `LCD_RESET` $\rightarrow$ `GPIO8`
  * Pin `27` (`LCD_TE`) $\rightarrow$ Net `LCD_TE` $\rightarrow$ `GPIO13`
* **Touch Panel (TP) Interface:**
  * Pin `3` (`TP_SCL`) $\rightarrow$ Net `TP_SCL` $\rightarrow$ `GPIO14` (Pull-up `R27` $10\text{ k}\Omega$ to `VCC3V3`).
  * Pin `5` (`TP_SDA`) $\rightarrow$ Net `TP_SDA` $\rightarrow$ `GPIO15` (Pull-up `R28` $10\text{ k}\Omega$ to `VCC3V3`).
  * Pin `7` (`TP_INT`) $\rightarrow$ Net `TP_INT` $\rightarrow$ `GPIO38`.
  * Pin `9` (`TP_RESET`) $\rightarrow$ Net `TP_RESET` $\rightarrow$ `GPIO9` (Pull-up `R26` $\text{NC}/10\text{ k}\Omega$).
* **Power Enable Control:**
  * Pin `21` (`DSI_PWR_EN`) $\rightarrow$ Resistor `R10` ($10\text{ k}\Omega$) $\rightarrow$ Rail `ALDO2`.

---

## 5. Power Management Unit (PMIC) - AXP2101

* **Primary Component:** `U1` (`AXP2101` PMIC)

### Regulator & Voltage Net Mapping:
* **`VBUS` Input:**
  * Pin `37` (`VBUS`) connected to `VBUS` input net via `CP1` ($10\,\mu\text{F}$) filter.
* **Battery Line:**
  * Pin `33` (`BAT`) connected to `VBAT1` through filter `CP5` ($2.2\,\mu\text{F}$) and Thermistor network `RP2` ($10\text{ k}\Omega$).
* **Buck Converter & LDO Outputs:**
  * Pin `21` (`LX1`) $\rightarrow$ Inductor `LP1` ($1\,\mu\text{H}$) $\rightarrow$ Rail `VCC3V3` (Feedback pin `22` `FB1`).
  * Pin `18` (`ALDO1`) $\rightarrow$ Rail `A3V3`.
  * Pin `19` (`ALDO2`) $\rightarrow$ Rail `VL2_3.3V`.
  * Pin `16` (`ALDO3`) $\rightarrow$ Rail `VCC3V`.
  * Pin `15` (`ALDO4`) $\rightarrow$ Rail `VL3_1.8V`.
  * Pin `12` (`BLDO1`) $\rightarrow$ Rail `VL_1.2V`.
  * Pin `14` (`BLDO2`) $\rightarrow$ Rail `VL_2.8V`.
  * Pin `28` (`RTCLDO`) $\rightarrow$ Rail `VCC-RTC` (Decoupled by `CP8` $2.2\,\mu\text{F}$).
* **Control & I2C Bus:**
  * Pin `39` (`SDA`) $\rightarrow$ Net `ESP32_SDA`.
  * Pin `40` (`SCK`) $\rightarrow$ Net `ESP32_SCL`.
  * Pin `38` (`IRQ`) $\rightarrow$ `EXIO5` / `AXP_IRQ`.
  * Pin `29` (`PWROK`) $\rightarrow$ Net `CHIP_PU`.
  * Pin `30` (`PWRON`) $\rightarrow$ Net `PWRON`.

---

## 6. Audio Codec (ES8311) & ADC (ES7210)

* **Primary Components:** `U1` (`ES8311` Codec), `U8` (`ES7210` ADC)

### ES8311 Audio Codec Pin Connections:
* **I2C Control Line:**
  * Pin `19` (`CE`) $\rightarrow$ Resistor `R50` ($10\text{ k}\Omega$) $\rightarrow$ `AGND`.
  * Pin `18` (`CDATA`) $\rightarrow$ Net `ESP32_SDA` (Pull-up `R23` $2.2\text{ k}\Omega$ to `VCC3V3`).
  * Pin `17` (`MIC1P/DMIC_SDA`) $\rightarrow$ Net `ESP32_SCL` (Pull-up `R49` $2.2\text{ k}\Omega$ to `VCC3V3`).
* **I2S Digital Audio Bus:**
  * Pin `1` (`CCLK`) $\rightarrow$ Net `I2S_MCLK` $\rightarrow$ `GPIO16`.
  * Pin `6` (`SCLK/DMIC_SCL`) $\rightarrow$ Net `I2S_SCLK` $\rightarrow$ `GPIO41`.
  * Pin `8` (`LRCK`) $\rightarrow$ Net `I2S_LRCK` $\rightarrow$ `GPIO45`.
  * Pin `9` (`DSDIN`) $\rightarrow$ Net `I2S_DSDIN` $\rightarrow$ `GPIO40`.
* **Analog Outputs:**
  * Pin `13` (`OUTN`) $\rightarrow$ Coupling Capacitor `C44` ($1\,\mu\text{F}$) $\rightarrow$ Net `OUTN`.
  * Pin `12` (`OUTP`) $\rightarrow$ Coupling Capacitor `C46` ($1\,\mu\text{F}$) $\rightarrow$ Net `OUTP`.

### ES7210 ADC Pin Connections:
* **I2S Audio Bus:**
  * Pin `9` (`SCLK`) $\rightarrow$ Net `I2S_SCLK`.
  * Pin `10` (`LRCK`) $\rightarrow$ Net `I2S_LRCK`.
  * Pin `11` (`SDOUT1/TDMOUT`) $\rightarrow$ `R47` ($51\Omega$) $\rightarrow$ Net `GPIO42_I2S_ASDOUT`.
* **Microphone Inputs:**
  * Pin `15` (`MIC1N`) $\rightarrow$ Capacitor `C99` ($1\,\mu\text{F}$) $\rightarrow$ Net `MIC1_N`.
  * Pin `16` (`MIC1P`) $\rightarrow$ Capacitor `C100` ($1\,\mu\text{F}$) $\rightarrow$ Net `MIC1_P`.
  * Pin `20` (`MIC2P`) $\rightarrow$ Capacitor `C104` ($1\,\mu\text{F}$) $\rightarrow$ Net `MIC2_P`.
  * Pin `21` (`MIC2N`) $\rightarrow$ Capacitor `C107` ($1\,\mu\text{F}$) $\rightarrow$ Net `MIC2_N`.

---

## 7. Audio Power Amplifier (NS4150B) & Speaker

* **Primary Component:** `U4` (`NS4150B` Class-D PA)

### Interconnections:
* **Control Input:**
  * Pin 1 (`CTRL`) $\rightarrow$ Resistor `R30` ($0\Omega$) $\rightarrow$ Net `GPIO46_PA_CTRL`.
  * Pin 1 $\rightarrow$ Pull-down Resistor `R31` ($10\text{ k}\Omega$) $\rightarrow$ `AGND`.
* **Differential Audio Inputs:**
  * Pin 3 (`IN+`) $\rightarrow$ Net `PA_IN+` $\rightarrow$ Resistor `R52` ($150\text{ k}\Omega$) $\rightarrow$ Filter $\rightarrow$ Net `OUTP`.
  * Pin 4 (`IN-`) $\rightarrow$ Net `PA_IN-` $\rightarrow$ Resistor `R53` ($150\text{ k}\Omega$) $\rightarrow$ Filter $\rightarrow$ Net `OUTN`.
* **Speaker Output Drive:**
  * Pin 8 (`OUT+`) $\rightarrow$ Net `PA_OUTL+` $\rightarrow$ Inductor `L4` ($0\Omega\text{ 0402}$) $\rightarrow$ Speaker Pad `P5`.
  * Pin 5 (`OUT-`) $\rightarrow$ Net `PA_OUTL-` $\rightarrow$ Inductor `L5` ($0\Omega\text{ 0402}$) $\rightarrow$ Speaker Pad `P6`.
* **Power Supply:**
  * Pin 6 (`VDD`) $\rightarrow$ Rail `VCC3V3` (Decoupling: `C55` $10\,\mu\text{F}$, `C56` $100\text{ nF}$, `C57` $1\,\mu\text{F}$ to `AGND`).

---

## 8. Microphones (MIC1 & MIC2)

* **Primary Components:** Analog/Digital MEMS Microphones `MIC1`, `MIC2`

### Interconnections:
* **Bias Power:**
  * Both `MIC1` and `MIC2` Pin `VDD` supplied by `ADC_MICBIAS12` rail.
  * Local Decoupling: `C58`/`C71` ($100\text{ nF}$) and `C54`/`C33` ($2.2\,\mu\text{F}$) to `AGND`.
* **Output Paths:**
  * `MIC1` Data output Pin 4 $\rightarrow$ Inductor `L6` ($0\Omega\text{ 0402}$) $\rightarrow$ `C53` ($1\,\mu\text{F}$) $\rightarrow$ Differential output lines `MIC1_P`, `MIC1_N`.
  * `MIC2` Data output Pin 4 $\rightarrow$ Inductor `L7` ($0\Omega\text{ 0402}$) $\rightarrow$ `C32` ($1\,\mu\text{F}$) $\rightarrow$ Differential output lines `MIC2_P`, `MIC2_N`.

---

## 9. Keys & MicroSD Interface

* **Primary Components:** Switches `Key1` (BOOT), `Key3` (PWR), MOSFET `T1` (`BSS138LT1G`), SD Connector `J4`

### Key Switches:
* **BOOT Button (`Key1`):**
  * Connects Net `GPIO0` to `GND` when actuated.
  * Pull-up: `R8` ($10\text{ k}\Omega$) to `VCC3V3`. Filter: `C14` ($100\text{ nF}$) to `GND`.
* **POWER Button (`Key3`):**
  * Connects Net `PWRON` to `GND` via `RP7` ($510\Omega$) when actuated.
* **Control Transistor (`T1`):**
  * Gate (Pin 1) $\rightarrow$ Resistor `R16` ($1\text{ k}\Omega$) $\rightarrow$ `PWR` switch node.
  * Drain (Pin 3) $\rightarrow$ Net `SYS_OUT` $\rightarrow$ Pull-up `R11` ($10\text{ k}\Omega$) to `VCC3V3`.
  * Source (Pin 2) $\rightarrow$ `GND`.

### MicroSD Card Socket (`J4`):
* Pin 1 (`D2`) $\rightarrow$ Unconnected / Reserved.
* Pin 2 (`CD/D3`) $\rightarrow$ Net `SDCS` $\rightarrow$ `GPIO17` (Pull-up `R3` $10\text{ k}\Omega$ to `VCC3V3`).
* Pin 3 (`CMD`) $\rightarrow$ Net `MOSI` $\rightarrow$ `GPIO1` (Pull-up `R4` $10\text{ k}\Omega$ to `VCC3V3`).
* Pin 4 (`VDD`) $\rightarrow$ Rail `VCC3V3` (Decoupling: `C3` $100\text{ nF}$, `C6` $2.2\,\mu\text{F}$).
* Pin 5 (`CLK`) $\rightarrow$ Net `SCK` $\rightarrow$ `GPIO2` (Pull-up `R5` $10\text{ k}\Omega$ to `VCC3V3`).
* Pin 6 (`VSS`) $\rightarrow$ `GND`.
* Pin 7 (`D0`) $\rightarrow$ Net `MISO` $\rightarrow$ `GPIO3` (Pull-up `R6` $10\text{ k}\Omega$ to `VCC3V3`).

---

## 10. Real-Time Clock (PCF85063ATL)

* **Primary Components:** `U6` (`PCF85063ATL`), Crystal `Y1` ($32.768\text{ kHz}$)

### Pin Interconnections:
* Pin 1 (`OSCI`), Pin 2 (`OSCO`) $\rightarrow$ Connected across crystal `Y1` with load capacitors `C22` ($22\text{ pF}$) and `C23` ($22\text{ pF}$) tied to `GND`.
* Pin 4 (`INT`) $\rightarrow$ Net `RTC_INT` $\rightarrow$ `GPIO39`.
* Pin 5 (`VSS`), Pin 11 (`EP`) $\rightarrow$ `GND`.
* Pin 6 (`SDA`) $\rightarrow$ Net `ESP32_SDA`.
* Pin 7 (`SCL`) $\rightarrow$ Net `ESP32_SCL`.
* Pin 10 (`VDD`) $\rightarrow$ Power Rail `VCC-RTC`.

---

## 11. 6/9-Axis Inertial Motion Sensor (QMI8658C)

* **Primary Component:** `U5` (`QMI8658C` IMU)

### Pin Interconnections:
* Pin 1 (`SDO/SAO`) $\rightarrow$ Connected to `GND` (Sets I2C address to `0x6B`).
* Pins 2 (`SDx`), 3 (`SCx`) $\rightarrow$ Tied together to `VCC3V3`.
* Pin 4 (`INT1`) $\rightarrow$ Net `QMI_INT1` $\rightarrow$ `GPIO21`.
* Pin 5 (`VDDIO`), Pin 8 (`VDD`) $\rightarrow$ Rail `VCC3V3` (Decoupling: `C26` $100\text{ nF}$ to `GND`).
* Pins 6 (`GND`), 7 (`GND`) $\rightarrow$ `GND`.
* Pin 9 (`INT2`) $\rightarrow$ Net `QMI_INT2` $\rightarrow$ Test point `TP15`.
* Pin 12 (`VCC3V3`) $\rightarrow$ `VCC3V3`.
* Pin 13 (`SCL`) $\rightarrow$ Net `ESP32_SCL`.
* Pin 14 (`SDA`) $\rightarrow$ Net `ESP32_SDA`.
```
eof