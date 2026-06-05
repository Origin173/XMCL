import { Box, Checkbox, Flex, Icon, Text } from "@chakra-ui/react";
import React, { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  LuChevronDown,
  LuChevronRight,
  LuFile,
  LuFolder,
} from "react-icons/lu";
import { ExportFileEntry, FileCategory } from "@/models/instance/misc";

interface TreeNode {
  entry: ExportFileEntry;
  children: TreeNode[];
}

function buildTree(entries: ExportFileEntry[]): TreeNode[] {
  const roots: TreeNode[] = [];
  const map: Record<string, TreeNode> = {};

  // Sort: directories first, then files
  const sorted = [...entries].sort((a, b) => {
    if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;
    return a.relativePath.localeCompare(b.relativePath);
  });

  for (const entry of sorted) {
    const node: TreeNode = { entry, children: [] };
    map[entry.relativePath] = node;

    const lastSlash = entry.relativePath.lastIndexOf("/");
    if (lastSlash === -1) {
      roots.push(node);
    } else {
      const parentPath = entry.relativePath.slice(0, lastSlash);
      if (map[parentPath]) {
        map[parentPath].children.push(node);
      } else {
        roots.push(node);
      }
    }
  }

  return roots;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Collect all leaf file paths under a node (including itself if it's a file). */
function collectFilePaths(node: TreeNode): string[] {
  if (!node.entry.isDirectory) return [node.entry.relativePath];
  return node.children.flatMap(collectFilePaths);
}

interface FileNodeProps {
  node: TreeNode;
  selected: Set<string>;
  onToggle: (paths: string[], checked: boolean) => void;
  depth: number;
}

const FileNode: React.FC<FileNodeProps> = ({
  node,
  selected,
  onToggle,
  depth,
}) => {
  const [expanded, setExpanded] = useState(false);

  const leafPaths = useMemo(() => collectFilePaths(node), [node]);

  const checkedCount = useMemo(
    () => leafPaths.filter((p) => selected.has(p)).length,
    [leafPaths, selected]
  );

  const isChecked = checkedCount === leafPaths.length && leafPaths.length > 0;
  const isIndeterminate = checkedCount > 0 && checkedCount < leafPaths.length;

  const handleCheck = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onToggle(leafPaths, e.target.checked);
    },
    [leafPaths, onToggle]
  );

  const isDir = node.entry.isDirectory;
  const label =
    node.entry.relativePath.split("/").pop() ?? node.entry.relativePath;
  const sizeLabel = !isDir ? formatBytes(node.entry.fileSize) : "";

  return (
    <Box>
      <Flex
        align="center"
        gap={1}
        pl={`${depth * 16 + 4}px`}
        py="2px"
        _hover={{ bg: "whiteAlpha.100" }}
        borderRadius="md"
        cursor="pointer"
      >
        <Checkbox
          isChecked={isChecked}
          isIndeterminate={isIndeterminate}
          onChange={handleCheck}
          size="sm"
          flexShrink={0}
          onClick={(e) => e.stopPropagation()}
        />
        {isDir && (
          <Icon
            as={expanded ? LuChevronDown : LuChevronRight}
            boxSize={3}
            flexShrink={0}
            cursor="pointer"
            onClick={() => setExpanded((v) => !v)}
          />
        )}
        {!isDir && <Box w={3} flexShrink={0} />}
        <Icon
          as={isDir ? LuFolder : LuFile}
          boxSize={3}
          color={isDir ? "yellow.400" : "gray.400"}
          flexShrink={0}
        />
        <Text
          fontSize="xs"
          flex={1}
          isTruncated
          cursor={isDir ? "pointer" : "default"}
          onClick={isDir ? () => setExpanded((v) => !v) : undefined}
          color={
            node.entry.category === FileCategory.Normal ? "gray.400" : undefined
          }
        >
          {label}
        </Text>
        {sizeLabel && (
          <Text fontSize="xs" color="gray.500" flexShrink={0} pr={2}>
            {sizeLabel}
          </Text>
        )}
      </Flex>

      {isDir &&
        expanded &&
        node.children.map((child) => (
          <FileNode
            key={child.entry.relativePath}
            node={child}
            selected={selected}
            onToggle={onToggle}
            depth={depth + 1}
          />
        ))}
    </Box>
  );
};

interface FileTreeSelectorProps {
  entries: ExportFileEntry[];
  selected: Set<string>;
  onChange: (selected: Set<string>) => void;
}

const FileTreeSelector: React.FC<FileTreeSelectorProps> = ({
  entries,
  selected,
  onChange,
}) => {
  const { t } = useTranslation();
  const tree = useMemo(() => buildTree(entries), [entries]);

  const handleToggle = useCallback(
    (paths: string[], checked: boolean) => {
      const next = new Set(selected);
      for (const p of paths) {
        if (checked) {
          next.add(p);
        } else {
          next.delete(p);
        }
      }
      onChange(next);
    },
    [selected, onChange]
  );

  if (tree.length === 0) {
    return (
      <Text fontSize="sm" color="gray.500" textAlign="center" py={4}>
        {t("FileTreeSelector.empty")}
      </Text>
    );
  }

  return (
    <Box overflowY="auto" maxH="360px" pr={1}>
      {tree.map((node) => (
        <FileNode
          key={node.entry.relativePath}
          node={node}
          selected={selected}
          onToggle={handleToggle}
          depth={0}
        />
      ))}
    </Box>
  );
};

export default FileTreeSelector;

/** Initialize default selected set based on FileCategory.
 * Suggested files are required pack content and are selected by default.
 * Normal files are visible but optional, so they start unselected.
 */
export function defaultSelectedFromEntries(
  entries: ExportFileEntry[]
): Set<string> {
  return new Set(
    entries
      .filter((e) => !e.isDirectory && e.category === FileCategory.Suggested)
      .map((e) => e.relativePath)
  );
}
