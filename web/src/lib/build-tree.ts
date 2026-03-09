/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { MailboxData } from "@/api/mailbox/api";
import { TreeDataItem } from "@/components/tree-view";
import { Badge } from '@/components/ui/badge'
import { FolderClosed, FolderOpen } from "lucide-react";
import React from 'react';

type BadgeContentFunction = (item: MailboxData) => React.ReactNode;

export const buildTree = (data: MailboxData[], badgeContent?: BadgeContentFunction, showAttributes?: boolean): TreeDataItem[] => {
    const root: TreeDataItem = {
        id: 'root',
        name: 'Root',
        icon: FolderClosed,
        openIcon: FolderOpen,
        children: [], // Ensure children is initialized as an array
    };

    const nodeMap: { [key: string]: TreeDataItem } = {};
    data.sort((a, b) => a.name.localeCompare(b.name));
    data.forEach((item) => {
        const { id, name, delimiter, exists, attributes } = item;
        const badge = badgeContent ? badgeContent(item) : React.createElement(Badge, {
            className: 'text-[12px]',
            variant: 'secondary',
        }, exists);


        const attributesNode = showAttributes
            ? attributes.map((item, index) =>
                React.createElement(
                    Badge,
                    {
                        key: index,
                        className: 'text-[12px] mr-1 last:mr-0',
                        variant: 'secondary',
                    },
                    item.attr === 'Extension' ? item.extension || '' : item.attr
                )
            )
            : null;
        // const badge = badgeContent || React.createElement(Badge, {
        //     className: 'text-xs',
        // }, exists);
        // If there is no delimiter, add the item directly as a child of the root
        if (!delimiter) {
            root.children!.push({
                id: id.toString(),
                name,
                icon: FolderClosed,
                openIcon: FolderOpen,
                badge,
                attributes: showAttributes ? attributesNode : null,
                children: undefined, // Leaf node, so children is undefined
            });
            return;
        }

        // Split the name into parts based on the delimiter
        const parts = name.split(delimiter); // dir/sub1/sub2
        let currentParent = root;

        // Traverse or create nodes for each part of the path
        parts.forEach((part, index) => {
            const path = parts.slice(0, index + 1).join(delimiter);

            // If the node already exists, set it as the current parent
            if (nodeMap[path]) {
                currentParent = nodeMap[path];
            } else {
                // Determine if this is a leaf node (last part of the path)
                const isLeaf = index === parts.length - 1;

                // Create a new node
                const newNode: TreeDataItem = {
                    id: isLeaf ? id.toString() : path, // Use item.id for leaf nodes, path for non-leaf nodes
                    name: part,
                    icon: FolderClosed,
                    openIcon: FolderOpen,
                    badge,
                    attributes: showAttributes ? attributesNode : null,
                    children: isLeaf ? undefined : [], // Ensure children is initialized as an array for non-leaf nodes
                };

                // Ensure currentParent.children is initialized as an array
                if (!currentParent.children) {
                    currentParent.children = [];
                }

                // Add the new node to the current parent's children
                currentParent.children.push(newNode);
                currentParent = newNode; // Update the current parent to the new node
                nodeMap[path] = newNode; // Store the node in the map for quick lookup
            }
        });
    });

    // Return the children of the root as the final tree structure
    return root.children!;
};