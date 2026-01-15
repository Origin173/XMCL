import { Button } from "@chakra-ui/react";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  OptionItemGroup,
  OptionItemGroupProps,
} from "@/components/common/option-item";
import { useLauncherConfig } from "@/contexts/config";
import { useSharedModals } from "@/contexts/shared-modal";
import { useToast } from "@/contexts/toast";
import { ConfigService } from "@/services/config";

const SyncAndRestoreSettingsPage = () => {
  const { t } = useTranslation();
  const { setConfig } = useLauncherConfig();
  const toast = useToast();
  const { openGenericConfirmDialog, closeSharedModal } = useSharedModals();

  const handleRestoreLauncherConfig = useCallback(async () => {
    ConfigService.restoreLauncherConfig().then((response) => {
      if (response.status === "success") {
        setConfig(response.data);
        toast({
          title: response.message,
          status: "success",
        });
      } else {
        toast({
          title: response.message,
          description: response.details,
          status: "error",
        });
      }
    });
    closeSharedModal("generic-confirm");
  }, [setConfig, toast, closeSharedModal]);

  const syncAndRestoreSettingGroups: OptionItemGroupProps[] = [
    {
      title: t("SyncAndRestoreSettingsPage.launcherConfig.title"),
      items: [
        {
          title: t(
            "SyncAndRestoreSettingsPage.launcherConfig.settings.restoreAll.title"
          ),
          description: t(
            "SyncAndRestoreSettingsPage.launcherConfig.settings.restoreAll.description"
          ),
          children: (
            <Button
              colorScheme="red"
              variant="subtle"
              size="xs"
              onClick={() => {
                openGenericConfirmDialog({
                  title: t("RestoreConfigConfirmDialog.title"),
                  body: t("RestoreConfigConfirmDialog.body"),
                  isAlert: true,
                  onOKCallback: handleRestoreLauncherConfig,
                });
              }}
            >
              {t(
                "SyncAndRestoreSettingsPage.launcherConfig.settings.restoreAll.restore"
              )}
            </Button>
          ),
        },
      ],
    },
  ];

  return (
    <>
      {syncAndRestoreSettingGroups.map((group, index) => (
        <OptionItemGroup title={group.title} items={group.items} key={index} />
      ))}
    </>
  );
};

export default SyncAndRestoreSettingsPage;
