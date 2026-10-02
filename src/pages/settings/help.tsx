import { Icon } from "@chakra-ui/react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useTranslation } from "react-i18next";
import { LuArrowRight } from "react-icons/lu";
import { CommonIconButton } from "@/components/common/common-icon-button";
import {
  OptionItemGroup,
  OptionItemGroupProps,
} from "@/components/common/option-item";
import { useSharedModals } from "@/contexts/shared-modal";

const HelpSettingsPage = () => {
  const { t } = useTranslation();
  const { openSharedModal } = useSharedModals();
  const helpSettingGroups: OptionItemGroupProps[] = [
    {
      items: [
        {
          title: t("HelpSettingsPage.top.settings.LauncherDocs.title"),
          children: (
            <CommonIconButton
              label={t("HelpSettingsPage.top.settings.LauncherDocs.url")}
              icon="external"
              withTooltip
              tooltipPlacement="bottom-end"
              size="xs"
              h={18}
              onClick={() =>
                openUrl(t("HelpSettingsPage.top.settings.LauncherDocs.url"))
              }
            />
          ),
        },
        {
          title: t("HelpSettingsPage.top.settings.bugReport.title"),
          description: t("HelpSettingsPage.top.settings.bugReport.description"),
          children: <Icon as={LuArrowRight} boxSize={3.5} mr="5px" />,
          isFullClickZone: true,
          onClick: () => openSharedModal("bug-report"),
        },
      ],
    },
    {
      title: t("HelpSettingsPage.minecraft.title"),
      items: [
        {
          title: t("HelpSettingsPage.minecraft.settings.mcWiki.title"),
          description: t(
            "HelpSettingsPage.minecraft.settings.mcWiki.description"
          ),
          children: (
            <CommonIconButton
              label={t("HelpSettingsPage.minecraft.settings.mcWiki.url")}
              icon="external"
              withTooltip
              tooltipPlacement="bottom-end"
              size="xs"
              onClick={() =>
                openUrl(t("HelpSettingsPage.minecraft.settings.mcWiki.url"))
              }
            />
          ),
        },
        {
          title: t("HelpSettingsPage.minecraft.settings.mcMod.title"),
          description: t(
            "HelpSettingsPage.minecraft.settings.mcMod.description"
          ),
          children: (
            <CommonIconButton
              label="https://www.mcmod.cn/"
              icon="external"
              withTooltip
              tooltipPlacement="bottom-end"
              size="xs"
              onClick={() => openUrl("https://www.mcmod.cn/")}
            />
          ),
        },
        {
          title: t("HelpSettingsPage.minecraft.settings.curseforge.title"),
          description: t(
            "HelpSettingsPage.minecraft.settings.curseforge.description"
          ),
          children: (
            <CommonIconButton
              label="https://www.curseforge.com/minecraft"
              icon="external"
              withTooltip
              tooltipPlacement="bottom-end"
              size="xs"
              onClick={() => openUrl("https://www.curseforge.com/minecraft")}
            />
          ),
        },
      ],
    },
    {
      title: t("HelpSettingsPage.community.title"),
      items: [
        {
          title: t("HelpSettingsPage.community.settings.MUA.title"),
          description: t("HelpSettingsPage.community.settings.MUA.description"),
          children: (
            <CommonIconButton
              label={t("HelpSettingsPage.community.settings.MUA.url")}
              icon="external"
              withTooltip
              tooltipPlacement="bottom-end"
              size="xs"
              onClick={() =>
                openUrl(t("HelpSettingsPage.community.settings.MUA.url"))
              }
            />
          ),
        },
        {
          title: t("HelpSettingsPage.community.settings.SJMC.title"),
          children: (
            <CommonIconButton
              label={t("HelpSettingsPage.community.settings.SJMC.url")}
              icon="external"
              withTooltip
              tooltipPlacement="bottom-end"
              size="xs"
              h={18}
              onClick={() =>
                openUrl(t("HelpSettingsPage.community.settings.SJMC.url"))
              }
            />
          ),
        },
      ],
    },
  ];

  return (
    <>
      {helpSettingGroups.map((group, index) => (
        <OptionItemGroup title={group.title} items={group.items} key={index} />
      ))}
    </>
  );
};

export default HelpSettingsPage;
