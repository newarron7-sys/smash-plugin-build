# Metal Sonic (REX) : voix et bruitages (15/11)

- Banque REX : `pf/sfx/18E.sawnd` (fichier 261 = 47 voix, fichier 629 = 118 sons). Ids des scripts (0A000100) :
  sons = id - 28641, voix = id - 28594 (formule de sons_chief.py : 0x4000 + (0x18E - 0x14B) * 0xA5 + 0x4B2).
  Vérifié : tous les ids de ses scripts tombent dans 0..117 / 0..46, et les ondes utilisées ne sont pas vides.
  (Fighter7D.dat n'est PAS sa fiche BrawlEx : 7D = Joker ; « 125 » était le n° de son module, pas son n° de perso.)
- Extraction : `sons-metal.yml` (Windows : sawnd -> DSP -> WAV -> IDSP), branche chiffrée `sons_metal`.
- Modèles des banques de Sonic (noms / ordre des pistes) : `modeles-sons.yml` + `rex/tools/modele_sons.py`.
  se : mod GameBanana 79820 « Non-specific ModernSFX for SonicR & Custom Moveset », fichier « se_sonic_c00 - ds ver1 -
  alot uses base sfx - frontiersjmp » (54 pistes, les 52 du jeu gardées) ; vc : mod 61889 (33 pistes, 32 gardées).
  À savoir : les pistes de sons non remplacées (atterrissages, accroupi, bord, boucle de la boule…) gardent le contenu
  de ce mod, pas celui du jeu. Le même mod a un fichier « basegenesis - nearvanilla » (plus proche du jeu) si besoin.
- Construction : `rex/tools/sons_metal.py` -> `sound/bank/fighter/se_sonic_c00.nus3audio` et
  `sound/bank/fighter_voice/vc_sonic_c00.nus3audio` de la source, renommés c08 par reslotter (les .nus3bank et
  .tonelabel restent partagés avec ceux du jeu). Toutes les voix de Sonic sont remplacées (REX ou silence).
- Lecture : `rex/metalsonic/src/sons.rs` (coups, prises, lancers, provocations, aux images de REX) et speciaux.rs.
  Son de charge du B neutre (n° 82, en boucle dans REX) : boucle retirée, lancé au début de la charge, arrêté à la
  décharge.

## Relevé de ses scripts (sous-action : image:S<n° son> / V<n° voix>)
```
 10 WalkSlow               18:S8 50:S9
 11 WalkMiddle             4:S8 19:S9
 12 WalkFast               3:S9 14:S8
 14 Dash                   0:S54
 16 RunBrake               3:S55
 17 Turn                   3:S8
 18 TurnRun                1:S55
 21 JumpF                  1:S63
 22 JumpF                  1:S63
 23 JumpB                  1:S63
 24 JumpB                  1:S63
 25 JumpAerialF            1:S64
 26 JumpAerialB            1:S64
 57 SpecialNShot           12:S82
 58 SpecialNShot           0:V3 0:S83 0:S84 0:S85 0.5:S86
 59 SpecialNShot           0:S93 0.875:V3 0.875:V7 0.875:S83
 67 EscapeN                1:S1
 68 EscapeF                4:S1
 69 EscapeB                4:S1
 70 EscapeAir              3:S1
 72 Attack11               3:S110
 73 Attack12               3:S108
 74 AttackS3S              6:S47
 78 AttackDash             4:S108
 79 AttackS3SNew           3:S110
 80 AttackS3SNew           3:S110
 83 AttackS3SNew           3:S110
 84 AttackHi3              3:S108
 85 AttackLw3              5:S107
 87 AttackS4S              11:V8 11:S109
 88 AttackS4S              11:V8 11:S109
 89 AttackLw32             6:S109
 90 AttackS4S              11:V8 11:S109
 93 AttackHi4              6:V7 6:V19 6:S3 6:S14
 96 AttackLw4              0:V9 0:V21 0:V26 1:S0
 99 AttackAirF             10:S109
100 AttackAirB             10:V8 10:S98
101 AttackAirHi            22:V8 23:S47
102 AttackAirLw            10:V9 10:S47
114 ThrowB                 41:S47
115 ThrowF                 18:V8
116 ThrowHi                24:S98
164 DamageFlyHi            0:V30 0:V31 0:V32 0:V33
165 DamageFlyN             0:V30 0:V31 0:V32 0:V33
166 DamageFlyLw            0:V30 0:V31 0:V32 0:V33
167 DamageFlyTop           0:V30 0:V31 0:V32 0:V33
168 DamageFlyRoll          0:V30 0:V31 0:V32 0:V33
176 DownAttackU            2:S10 18:S49 21:S49 46:S66
177 DownForwardU           0:S54
178 DownBackU              0:S54
184 DownStandD             0:S7
185 DownAttackD            2:S10 17:S49 45:S66
186 DownForwardD           0:S63
187 DownBackD              0:S63
191 PassiveStandF          3:S7
192 PassiveStandB          3:S7
193 PassiveWall            0:V42 0:S65
194 PassiveWallJump        0:S63
195 PassiveCeil            0:V42 0:S65
197 FuraFura               0:V34
202 FuraSleepLoop          0:V34 1:V34
207 OttottoWait            0:V41
213 CliffCatch             2:V28
215 CliffAttackQuick       23:S0 33:S105 48:S65
216 CliffClimbQuick        19:S105 23:S8 28:S9 31:S103
217 CliffEscapeQuick       2:S10 10:S54 34:S105 44:S66
218 CliffJumpQuick1        2:S10
219 CliffJumpQuick2        1:S63
220 CliffAttackSlow        40:S47 49:S47 61:S65
221 CliffClimbSlow         39:S105 52:S103
222 CliffEscapeSlow        48:S54 58:S105 70:S65
224 CliffJumpSlow2         2:S63
231 SlipAttack             17:S47 25:S47 38:S105 43:S103
232 SlipEscapeF            2:S7 22:S105 27:S103
233 SlipEscapeB            2:S7 21:S105 26:S103
239 SwimUp                 5:V43
252 HeavyWalk1             20:S8
253 HeavyWalk2             20:S8
298 Swing4Bat              35:V3
337 ItemLegsSlowF          9:S8 34:S8
338 ItemLegsMiddleF        3:S8 19:S8
339 ItemLegsFastF          4:S8 20:S8
340 ItemLegsBrakeF         10:S55
341 ItemLegsDashF          2:S54
342 ItemLegsSlowB          9:S8 34:S8
343 ItemLegsMiddleB        11:S8 26:S8
344 ItemLegsFastB          4:S8 21:S8
345 ItemLegsBrakeB         10:S55
346 ItemLegsDashB          2:S54
347 ItemLegsJumpSquat      1:S0
444 AppealHi               6:V46 35:V2
445 AppealHiR              6:V46 35:V2
446 AppealS                30:V5
447 AppealS                30:V5
448 AppealLw               8:V6 8:V20
449 AppealLwR              8:V6 8:V20
450 EntryR                 0:S54
451 EntryL                 0:S54
452 Win1                   0:S54 48:V13 48:V44
454 Win2                   0:S54 70:S55 80:V15 80:V45
456 Win3                   0:V4 0:V14 30:S49 70:S50
464 SpecialSStart          1:S97
465 SpecialSshort          0:V21 0:V39 1:S98
467 SpecialS               0:V21 0:V39 1:S98
468 SpecialAirS            0:V21 0:V39 1:S98
470 SpecialHiHold          1:S6
471 SpecialHiHoldAir       1:S6
472 SpecialHi              0:S11
476 SpecialLwStart         5:S87
477 SpecialLwLoop          26:S87
478 SpecialLwHit           0:S87 26:S87
480 SpecialAirLwStart      5:S87
481 SpecialLwLoop          26:S87
482 SpecialLwHit           0:S87 26:S87
484 FinalStart             36:S58 52:V3 70:S92 71:S92
485 FinalEnd               0:S58
487 FinalAirEnd            0:S58
```
