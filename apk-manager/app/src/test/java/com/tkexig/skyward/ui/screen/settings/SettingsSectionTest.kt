package com.tkexig.skyward.ui.screen.settings

import org.junit.Assert.assertEquals
import org.junit.Test

class SettingsSectionTest {

    @Test
    fun settingsSectionsResolvePreciseAliases() {
        assertEquals(SettingsSection.UI_MODE, SettingsSection.normalize("ui_mode"))
        assertEquals(SettingsSection.GENERAL, SettingsSection.cardFor("ui_mode"))
        assertEquals(SettingsSection.DIAGNOSTICS, SettingsSection.cardFor("diagnostics"))
        assertEquals(SettingsSection.UNINSTALL, SettingsSection.cardFor("uninstall"))
        assertEquals(SettingsSection.GENERAL, SettingsSection.normalize("not_a_section"))
    }
}
