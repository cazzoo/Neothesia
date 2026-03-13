# Plan d'Intégration ROLI LUMI Keys - Architecture Mise à Jour

## 📝 Objectif
Transformer Neothesia en une plateforme d'apprentissage interactive en exploitant le matériel ROLI LUMI Keys (LEDs par touche, mode "Wait", MPE).

---

## ✅ Phase Accomplie (Completed)

### Phase 1 : Communication SysEx & Protocole ✅
- [x] **Protocole SysEx ROLI** : Implémentation du packing de données 7-bits (BitArray) conforme aux spécifications ROLI
- [x] **Contrôleur LUMI (`lumi_controller.rs`)** : Création d'un module gérant l'allumage des LEDs RGB, la luminosité et les modes de couleur
- [x] **Tests Unitaires** : 9 tests unitaires vérifiant que les séquences SysEx générées correspondent exactement aux payloads attendus
- [x] **topologyIndex FIX** : Correction du bug topologyIndex (0x37 → 0x00) - **11 mars 2025**

### Phase 2 : Architecture & Connectivité ✅
- [x] **Sortie MIDI Dédiée** : Neothesia ouvre désormais un port de sortie MIDI "miroir" dédié au matériel LUMI
- [x] **Interface Paramètres (Menu)** : Ajout d'une section **LUMI Hardware** permettant de changer la luminosité et le mode d'éclairage
- [x] **Connection Management** : Détection automatique et connexion du clavier LUMI

### Phase 3 : Moteur de Jeu - Partiel ✅ ⚠️
- [x] **Wait Mode** : Intégration du système "PlayAlong" pour stopper le défilement tant que les touches attendues ne sont pas pressées
- [x] **Sound Triggering** : Sons joués immédiatement lors de l'appui sur les touches en mode Wait
- [x] **SysEx Delivery** : Messages SysEx atteignent le matériel avec le bon format
- [ ] **PANNE** : Hinting visuel ne fonctionne pas - clavier reste allumé pendant la lecture

---

## 🚨 Problème Critique Actuel (Critical Issue)

### Symptôme
Le clavier LUMI reste allumé avec les paramètres du menu pendant la lecture musicale, au lieu de:
1. S'éteindre (allumer touche par touche en foncé)
2. Montrer des hints (éclairage tamisé) 2 secondes avant l'arrivée des notes

### Diagnostic
Les messages SysEx atteignent le matériel (vérifié dans les logs), mais le clavier ne réagit pas visuellement.

**Hypothèse Principale** : Le mode de couleur (Rainbow/Piano/Night) doit être désactivé avant de pouvoir contrôler les LEDs individuellement pendant la lecture.

---

## 🛠 Architecture Requise - Deux Boucles Distinctes

### Architecture Actuelle (Défaillante)
```
Menu: set_color_mode(3) + set_brightness(50%)
  ↓
Playing: clear_all() + set_key_dim(hints)
  ↓
Résultat: Clavier ignore les commandes de hinting ❌
```

### Architecture Requise

#### Boucle 1: Menu Loop (Settings Mode) ✅ FONCTIONNE
**Objectif**: Afficher les préférences utilisateur
**Commandes**:
- `set_color_mode(mode)` - Mode (Rainbow/Single/Piano/Night)
- `set_brightness(percent)` - Luminosité 0-100%
**Contrôle**: Patterns硬件 intégrés du clavier
**Activation**: Scene de menu automatique
**Statut**: ✅ **TRAVAILLE**

#### Boucle 2: Song Loop (Manual Mode) ❌ À IMPLÉMENTER
**Objectif**: Guidance gameplay en temps réel
**Activation**:
```rust
fn enter_song_mode() {
    // Étape 1: Désactiver le mode de couleur automatique
    // TODO: Trouver la commande pour passer en contrôle manuel

    // Étape 2: Éteindre toutes les LEDs
    lumi.clear_all();

    // Étape 3: Activer le contrôle individuel des touches
    // TODO: Envoyer commande "enable manual per-key control"

    // Étape 4: Commencer le hinting
    // set_key_dim() pour les notes à venir
}
```

**Commandes**:
- `clear_all()` - Éteindre toutes les LEDs au démarrage
- `set_key_dim(note, r, g, b)` - Hints tamisés (60% luminosité)
- `set_key_color(note, r, g, b)` - Notes pressées (100% luminosité)

**Désactivation** (retour au menu):
```rust
fn exit_song_mode() {
    // Étape 1: Réinitialiser les LEDs
    lumi.clear_all();

    // Étape 2: Restaurer les paramètres du menu
    lumi.set_color_mode(ctx.config.lumi_color_mode());
    lumi.set_brightness(ctx.config.lumi_brightness());

    // Étape 3: Réactiver le mode automatique
    // TODO: Envier commande "enable color mode"
}
```

---

## 📋 Tâches Restantes (Implementation Tasks)

### Priorité Critique: Faire fonctionner le hinting ⚠️

#### Investigation Technique 🔍
1. **[ ] Comparer les byte sequences**:
   - Notre `clear_all()` vs référence `getColor(#000000)`
   - Vérifier si l'encodage est identique

2. **[ ] Rechercher la commande de changement de mode**:
   - Dans `.tmp-lumi/lumi-web-control/src/src/lumiSysexLib.js`
   - Chercher: "mode switch", "manual control", "direct LED"
   - Peut-être dans la documentation ROLI Blocks

3. **[ ] Tester la désactivation du color mode**:
   - Envoyer une commande spécifique avant `clear_all()`?
   - Ou `clear_all()` lui-même désactive-t-il le mode?

#### Implémentation 🔧

4. **[ ] Créer `enter_song_mode()`**:
   ```rust
   impl LumiController {
       pub fn enter_song_mode(&mut self) {
           // Désactiver le mode automatique
           // TODO: self.disable_color_mode();

           // Éteindre toutes les LEDs
           self.clear_all();

           // Activer le contrôle manuel par touche
           // TODO: self.enable_manual_control();
       }
   }
   ```

5. **[ ] Créer `exit_song_mode()`**:
   ```rust
   impl LumiController {
       pub fn exit_song_mode(&mut self, color_mode: u8, brightness: u8) {
           // Réinitialiser
           self.clear_all();

           // Restaurer les paramètres du menu
           self.set_color_mode(color_mode);
           self.set_brightness(brightness);

           // Réactiver le mode automatique
           // TODO: self.enable_color_mode();
       }
   }
   ```

6. **[ ] Intégrer dans PlayingScene**:
   ```rust
   // Dans playing_scene/mod.rs
   impl PlayingScene {
       fn new(ctx: &Context) -> Self {
           let mut lumi = LumiController::new(...);
           lumi.enter_song_mode();  // ← AJOUTER
           // ...
       }

       fn drop(&mut self) {
           self.lumi.exit_song_mode(...);  // ← AJOUTER
       }
   }
   ```

### Améliorations de l'Interface (UI) 📋
- [ ] **Visibilité Dynamique** : Masquer la section "LUMI Hardware" si aucun clavier détecté
- [ ] **Boutons Répétitifs** : Incrémentation continue en maintenant les boutons +/-
- [ ] **Écran de Transition** : Mode (Watch/Learn/Play) + sélection des mains

### Gameplay & MIDI 🎮
- [ ] **Masquage de Canaux** : Permettre de cacher certains canaux MIDI de la visualisation
- [ ] **Support MPE** : Aftertouch, pitch bend pour modulation des LEDs

### Finitions ✨
- [ ] **Écran de Score** : Résumé de performance (précision, timing)
- [ ] **Gestion des Soundfonts** : Cycle facile entre les soundfonts

---

## 🔬 Références Techniques

### Implementation de Référence
**`.tmp-lumi/lumi-web-control/src/src/lumiSysexLib.js`**
- `getColor(id, webColor)` - Contrôle par touche (id 0=primary, 1=root)
- `getBrightness(value)` - Luminosité globale 0-100
- `getColorMode(value)` - Mode (0=Rainbow, 1=Single, 2=Piano, 3=Night)

**LED Off Command (référence)**:
```javascript
// Noir = LEDs éteintes
Payload: [0x10, 0x20/0x30, 0x04, 0x00, 0x00, 0x00, 0x7E, 0xFF]
```

**Notre Implémentation Actuelle**:
```rust
// Envoie (0, 0, 0) à chaque touche 48-71
fn clear_all() {
    for note in 48..=71 {
        self.set_key_color(note, 0, 0, 0);
    }
}
```

### Documentation Externe
- [ROLI Blocks Protocol](https://github.com/WeAreROLI/roli_blocks_basics/tree/main/protocol)
- [benob/LUMI-lights](https://github.com/benob/LUMI-lights) - Reverse engineering initial
- [SYSEX.txt Reference](https://github.com/benob/LUMI-lights/blob/master/SYSEX.txt)

---

## 📊 Progression Globale

**Phase 1**: ✅ 100% (Protocole SysEx + Tests)
**Phase 2**: ✅ 100% (Architecture + Connectivité)
**Phase 3**: ⚠️ 40% (Wait Mode OK, Hinting en échec)
**UI & Polish**: 📋 20% (Paramètres OK, transition manquante)

**Statut Général**: 🟡 **FONCTIONNEL PARTIELLEMENT** - Le matériel communique mais le hinting nécessite une refactorisation de l'architecture de contrôle.

---

## 🎯 Prochaine Session Priorité

**IMMÉDIAT**: Comprendre pourquoi le mode de couleur bloque le contrôle individuel
1. Comparer les bytes de notre `clear_all()` avec la référence
2. Chercher une commande "mode switch" dans le code de référence
3. Tester l'envoi d'une commande de désactivation avant `clear_all()`

**COURT TERME**: Implémenter les deux boucles distinctes avec commutation de mode
**MOYEN TERME**: Finaliser UI et fonctionnalités gameplay
**LONG TERME**: Support MPE et fonctionnalités avancées
