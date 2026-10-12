**openOMSI {VERSION}: remote cockpit beta**

> [!WARNING]
> **Test build, expect bugs.** openOMSI needs an original OMSI 2 installation; it contains no game content. This build does **not** update itself: it stays on this version until you replace it.

**Drive your bus from a phone or tablet.** The game serves a page on your home network; open it on any phone or tablet (iPhone, iPad, Android) and you get:
- **a dashboard**: the map with your route, speed, next stop, the time, the bus's lamps
- **the bus's own buttons**: doors (every door, articulated buses too), lights, indicators, hazard lights, parking brake, engine start, kneeling...
- **the real destination sign** of the bus, live
- **a steering wheel and pedals** on the touch screen (two pedals, or one: up to accelerate, down to brake)
- **a layout editor**: move, resize, add or remove anything, per bus or for all

Nothing to install on the phone, no account, nothing leaves your network.

## Install

1. Download the file for your computer below (**Windows: `openOMSI-{VERSION}-windows-x64.zip`**).
2. Unzip it into a folder of its own that you can write to: Documents, a games folder... (not Program Files). It can live beside an official openOMSI: they share the settings.
3. Start it:
   - **Windows**: `openomsi.exe`. If SmartScreen warns about an unknown app: **More info → Run anyway**.
   - **macOS**: `openOMSI.app` (first time: right-click → Open → Open).
   - **Linux**: `./openomsi`
4. If it does not find OMSI 2 by itself, open **Setup** and pick your OMSI 2 folder (the one with `Omsi.exe`).

## Switch the cockpit on

1. Start a game (any map, any bus) and press **Esc**.
2. **Options → Interface → Remote cockpit (phone or tablet) → Open → Switch on.**
3. Connect the phone or tablet to **the same Wi-Fi** as the computer.
4. **Scan the QR code** with its camera: the cockpit opens and pairs by itself. Or type the address shown into its browser and enter the 6-digit code.
5. **Windows firewall**: the first time, Windows may ask whether openOMSI may use the network. Click **Allow** (at least *Private networks*), otherwise the phone cannot connect.

The device stays paired: next time, just open the page again (tip: *Add to Home Screen* makes it an app icon).

## What to test

Please try as much of this as you can, with **the buses you usually drive, mods included**:

- [ ] **Pairing**: QR code and typed code; a second phone or tablet at the same time
- [ ] **Dashboard**: speed, map and route, next stop, lamps in sync with the game
- [ ] **Doors**: each door button opens **its own** door, on buses with 2, 3, 4+ doors and articulated buses; *All doors*
- [ ] **Buttons**: lights, indicators, hazard lights, parking brake, engine start, kneeling, horn... do they do what they say?
- [ ] **Destination sign**: does the bus's real sign show (LED, roller blind, mod signs)? If not, does a plain drawn sign show instead of a black box?
- [ ] **Steering wheel and pedals** (third page): is steering smooth and precise? Pedals? Try *one pedal* in the editor (pencil icon → tap the pedals). Tap the **centre of the wheel** to bring it back to straight.
- [ ] **Layouts**: edit a page, save it for this bus or for all buses, switch bus and back
- [ ] **Phone and tablet, portrait and landscape**: is everything readable and easy to hit?
- [ ] **Robustness**: lock the phone and unlock it, leave Wi-Fi and come back, pause the game, change bus: does it reconnect and stay in sync? Does any button or pedal ever stay pressed?

**Known limits**: the phone and the computer must be on the same network. Tilting the phone to steer is not possible (browsers only allow it on secure pages). A bus whose own scripts are missing (some articulated Citaro mods) cannot open its doors from anywhere.

## Report what you find

On the [openOMSI Discord](https://discord.gg/VG2EKVafYG), say:
- the **bus** and **map**, the **phone or tablet** and its browser
- what you did, what you expected, what happened (a screenshot of the phone helps a lot)
- for a game problem, attach `game.log` from `C:\Users\<you>\.openomsi` (macOS/Linux: `~/.openomsi`)

## Downloads

| Platform | File |
| --- | --- |
| **Windows x64** | `openOMSI-{VERSION}-windows-x64.zip` |
| Windows ARM64 | `openOMSI-{VERSION}-windows-arm64.zip` |
| macOS (Apple silicon) | `openOMSI-{VERSION}-macos-arm64.zip` |
| macOS (Intel) | `openOMSI-{VERSION}-macos-x64.zip` |
| Linux x64 | `openOMSI-{VERSION}-linux-x64.zip` |
| Linux ARM64 | `openOMSI-{VERSION}-linux-arm64.zip` |

<details>
<summary><b>En français</b></summary>

**Piloter son bus depuis un téléphone ou une tablette** : tableau de bord avec la carte, les vrais boutons du bus, sa girouette en direct, un volant et des pédales tactiles, un éditeur de disposition.

**Installer** : télécharger le zip de son système (Windows : `openOMSI-{VERSION}-windows-x64.zip`), le décompresser dans un dossier à soi (pas Program Files), lancer `openomsi.exe` (SmartScreen : *Informations complémentaires → Exécuter quand même*), indiquer le dossier OMSI 2 si besoin. Cette version ne se met pas à jour toute seule.

**Activer** : lancer une partie, **Échap → Options → Interface → Cockpit à distance (téléphone ou tablette) → Ouvrir → Activer**. Téléphone sur **le même Wi-Fi**, scanner le QR code. Accepter l'autorisation du pare-feu Windows (réseaux privés).

**Tester** : la liste ci-dessus (appairage, tableau de bord, chaque porte ouvre la sienne, boutons, girouette, volant et pédales, dispositions, téléphone et tablette, verrouiller le téléphone ou changer de Wi-Fi), avec vos bus habituels, mods compris.

**Signaler** : sur le [Discord openOMSI](https://discord.gg/VG2EKVafYG), avec le bus, la carte, le téléphone et son navigateur, ce qui s'est passé (capture bienvenue) et `game.log` (`C:\Users\<vous>\.openomsi`) pour un souci côté jeu.

</details>
