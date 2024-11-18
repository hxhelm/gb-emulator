# GEDSE

A *G*ameboy *E*mulator that (hopefully) prepares me for N*DS* *E*mulation.

This is a WIP without a concrete idea in mind, just learning as I go.

## TODOs

### CPU Opcodes Implementation Checklist

<details>
<summary>Core Instructions (0x00-0xFF)</summary>

- [x] `0x00`, `NOP`: 1B, 4C, Flags: - - - -
- [x] `0x01`, `LD BC,n16`: 3B, 12C, Flags: - - - -
- [x] `0x02`, `LD (BC),A`: 1B, 8C, Flags: - - - -
- [x] `0x03`, `INC BC`: 1B, 8C, Flags: - - - -
- [x] `0x04`, `INC B`: 1B, 4C, Flags: Z 0 H -
- [x] `0x05`, `DEC B`: 1B, 4C, Flags: Z 1 H -
- [x] `0x06`, `LD B,n8`: 2B, 8C, Flags: - - - -
- [x] `0x07`, `RLCA`: 1B, 4C, Flags: 0 0 0 C
- [x] `0x08`, `LD (a16),SP`: 3B, 20C, Flags: - - - -
- [x] `0x09`, `ADD HL,BC`: 1B, 8C, Flags: - 0 H C
- [x] `0x0A`, `LD A,(BC)`: 1B, 8C, Flags: - - - -
- [x] `0x0B`, `DEC BC`: 1B, 8C, Flags: - - - -
- [x] `0x0C`, `INC C`: 1B, 4C, Flags: Z 0 H -
- [x] `0x0D`, `DEC C`: 1B, 4C, Flags: Z 1 H -
- [x] `0x0E`, `LD C,n8`: 2B, 8C, Flags: - - - -
- [x] `0x0F`, `RRCA`: 1B, 4C, Flags: 0 0 0 C
- [ ] `0x10`, `STOP n8`: 2B, 4C, Flags: - - - -
- [x] `0x11`, `LD DE,n16`: 3B, 12C, Flags: - - - -
- [x] `0x12`, `LD (DE),A`: 1B, 8C, Flags: - - - -
- [x] `0x13`, `INC DE`: 1B, 8C, Flags: - - - -
- [x] `0x14`, `INC D`: 1B, 4C, Flags: Z 0 H -
- [x] `0x15`, `DEC D`: 1B, 4C, Flags: Z 1 H -
- [x] `0x16`, `LD D,n8`: 2B, 8C, Flags: - - - -
- [x] `0x17`, `RLA`: 1B, 4C, Flags: 0 0 0 C
- [ ] `0x18`, `JR e8`: 2B, 12C, Flags: - - - -
- [x] `0x19`, `ADD HL,DE`: 1B, 8C, Flags: - 0 H C
- [x] `0x1A`, `LD A,(DE)`: 1B, 8C, Flags: - - - -
- [x] `0x1B`, `DEC DE`: 1B, 8C, Flags: - - - -
- [x] `0x1C`, `INC E`: 1B, 4C, Flags: Z 0 H -
- [x] `0x1D`, `DEC E`: 1B, 4C, Flags: Z 1 H -
- [x] `0x1E`, `LD E,n8`: 2B, 8C, Flags: - - - -
- [x] `0x1F`, `RRA`: 1B, 4C, Flags: 0 0 0 C
- [ ] `0x20`, `JR NZ,e8`: 2B, 12/8C, Flags: - - - -
- [x] `0x21`, `LD HL,n16`: 3B, 12C, Flags: - - - -
- [x] `0x22`, `LD (HL+),A`: 1B, 8C, Flags: - - - -
- [x] `0x23`, `INC HL`: 1B, 8C, Flags: - - - -
- [x] `0x24`, `INC H`: 1B, 4C, Flags: Z 0 H -
- [x] `0x25`, `DEC H`: 1B, 4C, Flags: Z 1 H -
- [x] `0x26`, `LD H,n8`: 2B, 8C, Flags: - - - -
- [x] `0x27`, `DAA`: 1B, 4C, Flags: Z - 0 C
- [ ] `0x28`, `JR Z,e8`: 2B, 12/8C, Flags: - - - -
- [x] `0x29`, `ADD HL,HL`: 1B, 8C, Flags: - 0 H C
- [x] `0x2A`, `LD A,(HL+)`: 1B, 8C, Flags: - - - -
- [x] `0x2B`, `DEC HL`: 1B, 8C, Flags: - - - -
- [x] `0x2C`, `INC L`: 1B, 4C, Flags: Z 0 H -
- [x] `0x2D`, `DEC L`: 1B, 4C, Flags: Z 1 H -
- [x] `0x2E`, `LD L,n8`: 2B, 8C, Flags: - - - -
- [x] `0x2F`, `CPL`: 1B, 4C, Flags: - 1 1 -
- [ ] `0x30`, `JR NC,e8`: 2B, 12/8C, Flags: - - - -
- [x] `0x31`, `LD SP,n16`: 3B, 12C, Flags: - - - -
- [x] `0x32`, `LD (HL-),A`: 1B, 8C, Flags: - - - -
- [x] `0x33`, `INC SP`: 1B, 8C, Flags: - - - -
- [x] `0x34`, `INC (HL)`: 1B, 12C, Flags: Z 0 H -
- [x] `0x35`, `DEC (HL)`: 1B, 12C, Flags: Z 1 H -
- [x] `0x36`, `LD (HL),n8`: 2B, 12C, Flags: - - - -
- [x] `0x37`, `SCF`: 1B, 4C, Flags: - 0 0 1
- [ ] `0x38`, `JR C,e8`: 2B, 12/8C, Flags: - - - -
- [x] `0x39`, `ADD HL,SP`: 1B, 8C, Flags: - 0 H C
- [x] `0x3A`, `LD A,(HL-)`: 1B, 8C, Flags: - - - -
- [x] `0x3B`, `DEC SP`: 1B, 8C, Flags: - - - -
- [x] `0x3C`, `INC A`: 1B, 4C, Flags: Z 0 H -
- [x] `0x3D`, `DEC A`: 1B, 4C, Flags: Z 1 H -
- [x] `0x3E`, `LD A,n8`: 2B, 8C, Flags: - - - -
- [x] `0x3F`, `CCF`: 1B, 4C, Flags: - 0 0 C
- [x] `0x40`, `LD B,B`: 1B, 4C, Flags: - - - -
- [x] `0x41`, `LD B,C`: 1B, 4C, Flags: - - - -
- [x] `0x42`, `LD B,D`: 1B, 4C, Flags: - - - -
- [x] `0x43`, `LD B,E`: 1B, 4C, Flags: - - - -
- [x] `0x44`, `LD B,H`: 1B, 4C, Flags: - - - -
- [x] `0x45`, `LD B,L`: 1B, 4C, Flags: - - - -
- [x] `0x46`, `LD B,(HL)`: 1B, 8C, Flags: - - - -
- [x] `0x47`, `LD B,A`: 1B, 4C, Flags: - - - -
- [x] `0x48`, `LD C,B`: 1B, 4C, Flags: - - - -
- [x] `0x49`, `LD C,C`: 1B, 4C, Flags: - - - -
- [x] `0x4A`, `LD C,D`: 1B, 4C, Flags: - - - -
- [x] `0x4B`, `LD C,E`: 1B, 4C, Flags: - - - -
- [x] `0x4C`, `LD C,H`: 1B, 4C, Flags: - - - -
- [x] `0x4D`, `LD C,L`: 1B, 4C, Flags: - - - -
- [x] `0x4E`, `LD C,(HL)`: 1B, 8C, Flags: - - - -
- [x] `0x4F`, `LD C,A`: 1B, 4C, Flags: - - - -
- [x] `0x50`, `LD D,B`: 1B, 4C, Flags: - - - -
- [x] `0x51`, `LD D,C`: 1B, 4C, Flags: - - - -
- [x] `0x52`, `LD D,D`: 1B, 4C, Flags: - - - -
- [x] `0x53`, `LD D,E`: 1B, 4C, Flags: - - - -
- [x] `0x54`, `LD D,H`: 1B, 4C, Flags: - - - -
- [x] `0x55`, `LD D,L`: 1B, 4C, Flags: - - - -
- [x] `0x56`, `LD D,(HL)`: 1B, 8C, Flags: - - - -
- [x] `0x57`, `LD D,A`: 1B, 4C, Flags: - - - -
- [x] `0x58`, `LD E,B`: 1B, 4C, Flags: - - - -
- [x] `0x59`, `LD E,C`: 1B, 4C, Flags: - - - -
- [x] `0x5A`, `LD E,D`: 1B, 4C, Flags: - - - -
- [x] `0x5B`, `LD E,E`: 1B, 4C, Flags: - - - -
- [x] `0x5C`, `LD E,H`: 1B, 4C, Flags: - - - -
- [x] `0x5D`, `LD E,L`: 1B, 4C, Flags: - - - -
- [x] `0x5E`, `LD E,(HL)`: 1B, 8C, Flags: - - - -
- [x] `0x5F`, `LD E,A`: 1B, 4C, Flags: - - - -
- [x] `0x60`, `LD H,B`: 1B, 4C, Flags: - - - -
- [x] `0x61`, `LD H,C`: 1B, 4C, Flags: - - - -
- [x] `0x62`, `LD H,D`: 1B, 4C, Flags: - - - -
- [x] `0x63`, `LD H,E`: 1B, 4C, Flags: - - - -
- [x] `0x64`, `LD H,H`: 1B, 4C, Flags: - - - -
- [x] `0x65`, `LD H,L`: 1B, 4C, Flags: - - - -
- [x] `0x66`, `LD H,(HL)`: 1B, 8C, Flags: - - - -
- [x] `0x67`, `LD H,A`: 1B, 4C, Flags: - - - -
- [x] `0x68`, `LD L,B`: 1B, 4C, Flags: - - - -
- [x] `0x69`, `LD L,C`: 1B, 4C, Flags: - - - -
- [x] `0x6A`, `LD L,D`: 1B, 4C, Flags: - - - -
- [x] `0x6B`, `LD L,E`: 1B, 4C, Flags: - - - -
- [x] `0x6C`, `LD L,H`: 1B, 4C, Flags: - - - -
- [x] `0x6D`, `LD L,L`: 1B, 4C, Flags: - - - -
- [x] `0x6E`, `LD L,(HL)`: 1B, 8C, Flags: - - - -
- [x] `0x6F`, `LD L,A`: 1B, 4C, Flags: - - - -
- [x] `0x70`, `LD (HL),B`: 1B, 8C, Flags: - - - -
- [x] `0x71`, `LD (HL),C`: 1B, 8C, Flags: - - - -
- [x] `0x72`, `LD (HL),D`: 1B, 8C, Flags: - - - -
- [x] `0x73`, `LD (HL),E`: 1B, 8C, Flags: - - - -
- [x] `0x74`, `LD (HL),H`: 1B, 8C, Flags: - - - -
- [x] `0x75`, `LD (HL),L`: 1B, 8C, Flags: - - - -
- [ ] `0x76`, `HALT`: 1B, 4C, Flags: - - - -
- [x] `0x77`, `LD (HL),A`: 1B, 8C, Flags: - - - -
- [x] `0x78`, `LD A,B`: 1B, 4C, Flags: - - - -
- [x] `0x79`, `LD A,C`: 1B, 4C, Flags: - - - -
- [x] `0x7A`, `LD A,D`: 1B, 4C, Flags: - - - -
- [x] `0x7B`, `LD A,E`: 1B, 4C, Flags: - - - -
- [x] `0x7C`, `LD A,H`: 1B, 4C, Flags: - - - -
- [x] `0x7D`, `LD A,L`: 1B, 4C, Flags: - - - -
- [x] `0x7E`, `LD A,(HL)`: 1B, 8C, Flags: - - - -
- [x] `0x7F`, `LD A,A`: 1B, 4C, Flags: - - - -
- [x] `0x80`, `ADD A,B`: 1B, 4C, Flags: Z 0 H C
- [x] `0x81`, `ADD A,C`: 1B, 4C, Flags: Z 0 H C
- [x] `0x82`, `ADD A,D`: 1B, 4C, Flags: Z 0 H C
- [x] `0x83`, `ADD A,E`: 1B, 4C, Flags: Z 0 H C
- [x] `0x84`, `ADD A,H`: 1B, 4C, Flags: Z 0 H C
- [x] `0x85`, `ADD A,L`: 1B, 4C, Flags: Z 0 H C
- [x] `0x86`, `ADD A,(HL)`: 1B, 8C, Flags: Z 0 H C
- [x] `0x87`, `ADD A,A`: 1B, 4C, Flags: Z 0 H C
- [x] `0x88`, `ADC A,B`: 1B, 4C, Flags: Z 0 H C
- [x] `0x89`, `ADC A,C`: 1B, 4C, Flags: Z 0 H C
- [x] `0x8A`, `ADC A,D`: 1B, 4C, Flags: Z 0 H C
- [x] `0x8B`, `ADC A,E`: 1B, 4C, Flags: Z 0 H C
- [x] `0x8C`, `ADC A,H`: 1B, 4C, Flags: Z 0 H C
- [x] `0x8D`, `ADC A,L`: 1B, 4C, Flags: Z 0 H C
- [x] `0x8E`, `ADC A,(HL)`: 1B, 8C, Flags: Z 0 H C
- [x] `0x8F`, `ADC A,A`: 1B, 4C, Flags: Z 0 H C
- [x] `0x90`, `SUB A,B`: 1B, 4C, Flags: Z 1 H C
- [x] `0x91`, `SUB A,C`: 1B, 4C, Flags: Z 1 H C
- [x] `0x92`, `SUB A,D`: 1B, 4C, Flags: Z 1 H C
- [x] `0x93`, `SUB A,E`: 1B, 4C, Flags: Z 1 H C
- [x] `0x94`, `SUB A,H`: 1B, 4C, Flags: Z 1 H C
- [x] `0x95`, `SUB A,L`: 1B, 4C, Flags: Z 1 H C
- [x] `0x96`, `SUB A,(HL)`: 1B, 8C, Flags: Z 1 H C
- [x] `0x97`, `SUB A,A`: 1B, 4C, Flags: 1 1 0 0
- [x] `0x98`, `SBC A,B`: 1B, 4C, Flags: Z 1 H C
- [x] `0x99`, `SBC A,C`: 1B, 4C, Flags: Z 1 H C
- [x] `0x9A`, `SBC A,D`: 1B, 4C, Flags: Z 1 H C
- [x] `0x9B`, `SBC A,E`: 1B, 4C, Flags: Z 1 H C
- [x] `0x9C`, `SBC A,H`: 1B, 4C, Flags: Z 1 H C
- [x] `0x9D`, `SBC A,L`: 1B, 4C, Flags: Z 1 H C
- [x] `0x9E`, `SBC A,(HL)`: 1B, 8C, Flags: Z 1 H C
- [x] `0x9F`, `SBC A,A`: 1B, 4C, Flags: Z 1 H -
- [x] `0xA0`, `AND A,B`: 1B, 4C, Flags: Z 0 1 0
- [x] `0xA1`, `AND A,C`: 1B, 4C, Flags: Z 0 1 0
- [x] `0xA2`, `AND A,D`: 1B, 4C, Flags: Z 0 1 0
- [x] `0xA3`, `AND A,E`: 1B, 4C, Flags: Z 0 1 0
- [x] `0xA4`, `AND A,H`: 1B, 4C, Flags: Z 0 1 0
- [x] `0xA5`, `AND A,L`: 1B, 4C, Flags: Z 0 1 0
- [x] `0xA6`, `AND A,(HL)`: 1B, 8C, Flags: Z 0 1 0
- [x] `0xA7`, `AND A,A`: 1B, 4C, Flags: Z 0 1 0
- [x] `0xA8`, `XOR A,B`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xA9`, `XOR A,C`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xAA`, `XOR A,D`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xAB`, `XOR A,E`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xAC`, `XOR A,H`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xAD`, `XOR A,L`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xAE`, `XOR A,(HL)`: 1B, 8C, Flags: Z 0 0 0
- [x] `0xAF`, `XOR A,A`: 1B, 4C, Flags: 1 0 0 0
- [x] `0xB0`, `OR A,B`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xB1`, `OR A,C`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xB2`, `OR A,D`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xB3`, `OR A,E`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xB4`, `OR A,H`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xB5`, `OR A,L`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xB6`, `OR A,(HL)`: 1B, 8C, Flags: Z 0 0 0
- [x] `0xB7`, `OR A,A`: 1B, 4C, Flags: Z 0 0 0
- [x] `0xB8`, `CP A,B`: 1B, 4C, Flags: Z 1 H C
- [x] `0xB9`, `CP A,C`: 1B, 4C, Flags: Z 1 H C
- [x] `0xBA`, `CP A,D`: 1B, 4C, Flags: Z 1 H C
- [x] `0xBB`, `CP A,E`: 1B, 4C, Flags: Z 1 H C
- [x] `0xBC`, `CP A,H`: 1B, 4C, Flags: Z 1 H C
- [x] `0xBD`, `CP A,L`: 1B, 4C, Flags: Z 1 H C
- [x] `0xBE`, `CP A,(HL)`: 1B, 8C, Flags: Z 1 H C
- [x] `0xBF`, `CP A,A`: 1B, 4C, Flags: 1 1 0 0
- [ ] `0xC0`, `RET NZ`: 1B, 20/8C, Flags: - - - -
- [ ] `0xC1`, `POP BC`: 1B, 12C, Flags: - - - -
- [ ] `0xC2`, `JP NZ,a16`: 3B, 16/12C, Flags: - - - -
- [ ] `0xC3`, `JP a16`: 3B, 16C, Flags: - - - -
- [ ] `0xC4`, `CALL NZ,a16`: 3B, 24/12C, Flags: - - - -
- [ ] `0xC5`, `PUSH BC`: 1B, 16C, Flags: - - - -
- [x] `0xC6`, `ADD A,n8`: 2B, 8C, Flags: Z 0 H C
- [ ] `0xC7`, `RST $00`: 1B, 16C, Flags: - - - -
- [ ] `0xC8`, `RET Z`: 1B, 20/8C, Flags: - - - -
- [ ] `0xC9`, `RET`: 1B, 16C, Flags: - - - -
- [ ] `0xCA`, `JP Z,a16`: 3B, 16/12C, Flags: - - - -
- [ ] `0xCB`, `PREFIX CB`: 1B, 4C, Flags: - - - -
- [ ] `0xCC`, `CALL Z,a16`: 3B, 24/12C, Flags: - - - -
- [ ] `0xCD`, `CALL a16`: 3B, 24C, Flags: - - - -
- [x] `0xCE`, `ADC A,n8`: 2B, 8C, Flags: Z 0 H C
- [ ] `0xCF`, `RST $08`: 1B, 16C, Flags: - - - -
- [ ] `0xD0`, `RET NC`: 1B, 20/8C, Flags: - - - -
- [ ] `0xD1`, `POP DE`: 1B, 12C, Flags: - - - -
- [ ] `0xD2`, `JP NC,a16`: 3B, 16/12C, Flags: - - - -
- [ ] `0xD4`, `CALL NC,a16`: 3B, 24/12C, Flags: - - - -
- [ ] `0xD5`, `PUSH DE`: 1B, 16C, Flags: - - - -
- [x] `0xD6`, `SUB A,n8`: 2B, 8C, Flags: Z 1 H C
- [ ] `0xD7`, `RST $10`: 1B, 16C, Flags: - - - -
- [ ] `0xD8`, `RET C`: 1B, 20/8C, Flags: - - - -
- [ ] `0xD9`, `RETI`: 1B, 16C, Flags: - - - -
- [ ] `0xDA`, `JP C,a16`: 3B, 16/12C, Flags: - - - -
- [ ] `0xDC`, `CALL C,a16`: 3B, 24/12C, Flags: - - - -
- [x] `0xDE`, `SBC A,n8`: 2B, 8C, Flags: Z 1 H C
- [ ] `0xDF`, `RST $18`: 1B, 16C, Flags: - - - -
- [x] `0xE0`, `LDH (a8),A`: 2B, 12C, Flags: - - - -
- [ ] `0xE1`, `POP HL`: 1B, 12C, Flags: - - - -
- [x] `0xE2`, `LD (C),A`: 1B, 8C, Flags: - - - -
- [ ] `0xE5`, `PUSH HL`: 1B, 16C, Flags: - - - -
- [x] `0xE6`, `AND A,n8`: 2B, 8C, Flags: Z 0 1 0
- [ ] `0xE7`, `RST $20`: 1B, 16C, Flags: - - - -
- [x] `0xE8`, `ADD SP,e8`: 2B, 16C, Flags: 0 0 H C
- [ ] `0xE9`, `JP HL`: 1B, 4C, Flags: - - - -
- [x] `0xEA`, `LD (a16),A`: 3B, 16C, Flags: - - - -
- [x] `0xEE`, `XOR A,n8`: 2B, 8C, Flags: Z 0 0 0
- [ ] `0xEF`, `RST $28`: 1B, 16C, Flags: - - - -
- [x] `0xF0`, `LDH A,(a8)`: 2B, 12C, Flags: - - - -
- [ ] `0xF1`, `POP AF`: 1B, 12C, Flags: Z N H C
- [x] `0xF2`, `LD A,(C)`: 1B, 8C, Flags: - - - -
- [x] `0xF3`, `DI`: 1B, 4C, Flags: - - - -
- [ ] `0xF5`, `PUSH AF`: 1B, 16C, Flags: - - - -
- [x] `0xF6`, `OR A,n8`: 2B, 8C, Flags: Z 0 0 0
- [ ] `0xF7`, `RST $30`: 1B, 16C, Flags: - - - -
- [x] `0xF8`, `LD HL,SP+e8`: 2B, 12C, Flags: 0 0 H C
- [x] `0xF9`, `LD SP,HL`: 1B, 8C, Flags: - - - -
- [x] `0xFA`, `LD A,(a16)`: 3B, 16C, Flags: - - - -
- [x] `0xFB`, `EI`: 1B, 4C, Flags: - - - -
- [x] `0xFE`, `CP A,n8`: 2B, 8C, Flags: Z 1 H C
- [ ] `0xEF`, `RST $38`: 1B, 16C, Flags: - - - -
</details>

