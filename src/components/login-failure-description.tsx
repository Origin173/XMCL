import { Link, Text } from "@chakra-ui/react";
import { t } from "i18next";
import React from "react";

interface LoginFailureDescriptionProps {
  details?: string;
  onClearCache: () => void;
}

// Toasts are rendered outside the app's context providers, so the caller passes in the callback.
const LoginFailureDescription: React.FC<LoginFailureDescriptionProps> = ({
  details,
  onClearCache,
}) => (
  <>
    {details && <Text>{details}</Text>}
    <Text>
      {t("LoginFailureDescription.hint")}
      <Link textDecoration="underline" onClick={onClearCache}>
        {t("LoginFailureDescription.clearCache")}
      </Link>
    </Text>
  </>
);

export const LOGIN_CACHE_ONLY_OPTIONS = {
  login: true,
  download: false,
  temp: false,
  logs: false,
};

export const LOGIN_FAILURE_TOAST_DURATION = 8000;

export default LoginFailureDescription;
