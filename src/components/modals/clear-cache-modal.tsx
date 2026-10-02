import {
  Button,
  Checkbox,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalFooter,
  ModalHeader,
  ModalOverlay,
  ModalProps,
  Text,
  VStack,
} from "@chakra-ui/react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useLauncherConfig } from "@/contexts/config";
import { useGlobalData } from "@/contexts/global-data";
import { useToast } from "@/contexts/toast";
import { ClearCacheOptions } from "@/models/config";
import { ConfigService } from "@/services/config";
import { formatByteSize } from "@/utils/string";

interface ClearCacheModalProps extends Omit<ModalProps, "children"> {
  initialOptions?: Partial<ClearCacheOptions>;
}

const defaultOptions: ClearCacheOptions = {
  login: true,
  credentials: false,
  download: true,
  temp: true,
  logs: false,
};

const optionKeys: (keyof ClearCacheOptions)[] = [
  "login",
  "download",
  "temp",
  "logs",
  "credentials",
];

const ClearCacheModal: React.FC<ClearCacheModalProps> = ({
  initialOptions,
  ...modalProps
}) => {
  const { t } = useTranslation();
  const toast = useToast();
  const { config } = useLauncherConfig();
  const primaryColor = config.appearance.theme.primaryColor;
  const { getPlayerList, getAuthServerList } = useGlobalData();

  const [options, setOptions] = useState<ClearCacheOptions>({
    ...defaultOptions,
    ...initialOptions,
  });
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const handleClear = async () => {
    setIsLoading(true);
    const response = await ConfigService.clearLauncherCache(options);
    setIsLoading(false);

    if (response.status !== "success") {
      toast({
        title: response.message,
        description: response.details,
        status: "error",
      });
      return;
    }

    const { freedBytes, failedCount } = response.data;
    toast({
      title: response.message,
      description:
        t("ClearCacheModal.result.freed", {
          size: formatByteSize(freedBytes),
        }) +
        (failedCount > 0
          ? t("ClearCacheModal.result.failed", { count: failedCount })
          : ""),
      status: failedCount > 0 ? "warning" : "success",
    });

    if (options.login) {
      getPlayerList(true);
      getAuthServerList(true);
    }
    modalProps.onClose();
  };

  return (
    <Modal size={{ base: "md", lg: "lg" }} {...modalProps}>
      <ModalOverlay />
      <ModalContent>
        <ModalHeader>{t("ClearCacheModal.header")}</ModalHeader>
        <ModalCloseButton />
        <ModalBody>
          <VStack align="stretch" spacing={3}>
            <Text fontSize="sm">{t("ClearCacheModal.body")}</Text>
            {optionKeys.map((key) => (
              <Checkbox
                key={key}
                colorScheme={key === "credentials" ? "red" : primaryColor}
                isChecked={options[key]}
                isDisabled={key === "credentials" && !options.login}
                onChange={(e) =>
                  setOptions((prev) => ({
                    ...prev,
                    [key]: e.target.checked,
                    ...(key === "login" && !e.target.checked
                      ? { credentials: false }
                      : {}),
                  }))
                }
                alignItems="flex-start"
                pl={key === "credentials" ? 6 : 0}
              >
                <Text fontSize="sm" mt={-0.5}>
                  {t(`ClearCacheModal.options.${key}.title`)}
                </Text>
                <Text fontSize="xs" className="secondary-text">
                  {t(`ClearCacheModal.options.${key}.description`)}
                </Text>
              </Checkbox>
            ))}
          </VStack>
        </ModalBody>
        <ModalFooter>
          <Button variant="ghost" onClick={modalProps.onClose}>
            {t("General.cancel")}
          </Button>
          <Button
            colorScheme={options.credentials ? "red" : primaryColor}
            onClick={handleClear}
            isLoading={isLoading}
            isDisabled={!optionKeys.some((key) => options[key])}
          >
            {t("ClearCacheModal.button.clear")}
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
};

export default ClearCacheModal;
