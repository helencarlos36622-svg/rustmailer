/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react'
import * as AccordionPrimitive from '@radix-ui/react-accordion'
import { ChevronRight } from 'lucide-react'
import { cva } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const treeVariants = cva(
    'group hover:before:opacity-100 before:absolute before:rounded-lg before:left-0 px-2 before:w-full before:opacity-0 before:bg-accent/70 before:h-[2rem] before:-z-10'
)

const selectedTreeVariants = cva(
    'before:opacity-100 before:bg-accent/70 text-accent-foreground'
)

interface TreeDataItem {
    id: string
    name: string
    icon?: React.ComponentType<{ className?: string }>
    openIcon?: React.ComponentType<{ className?: string }>
    children?: TreeDataItem[]
    badge?: React.ReactNode,
    attributes?: React.ReactNode,
    onClick?: () => void
}

type TreeProps = React.HTMLAttributes<HTMLDivElement> & {
    data: TreeDataItem[] | TreeDataItem
    onSelectChange?: (item: TreeDataItem | undefined) => void
    onSelectItemsChange?: (items: TreeDataItem[]) => void
    expandAll?: boolean
    multiple?: boolean
    clickRowToSelect?: boolean
    initialSelectedItemIds?: string[]
    defaultNodeIcon?: React.ComponentType<{ className?: string }>
    defaultLeafIcon?: React.ComponentType<{ className?: string }>
}

const TreeView = React.forwardRef<HTMLDivElement, TreeProps>(
    (
        {
            data,
            onSelectChange,
            onSelectItemsChange,
            expandAll,
            defaultLeafIcon,
            defaultNodeIcon,
            clickRowToSelect = true,
            className,
            multiple,
            initialSelectedItemIds = [],
            ...props
        },
        ref
    ) => {
        const [selectedItemIds, setSelectedItemIds] = React.useState<Set<string>>(
            new Set(initialSelectedItemIds)
        );

        const callbacksRef = React.useRef({
            onSelectChange,
            onSelectItemsChange
        });

        React.useEffect(() => {
            callbacksRef.current = {
                onSelectChange,
                onSelectItemsChange
            };
        }, [onSelectChange, onSelectItemsChange]);


        const handleSelectChange = React.useCallback(
            (item: TreeDataItem | undefined) => {
                if (!item) return;

                setSelectedItemIds(prev => {
                    const newSet = new Set(prev);
                    if (newSet.has(item.id)) {
                        newSet.delete(item.id);
                    } else {
                        if (!multiple) {
                            newSet.clear();
                        }
                        newSet.add(item.id);
                    }

                    setTimeout(() => {
                        if (callbacksRef.current.onSelectChange) {
                            callbacksRef.current.onSelectChange(newSet.has(item.id) ? item : undefined);
                        }
                        if (callbacksRef.current.onSelectItemsChange) {
                            const selectedItems = Array.from(newSet)
                                .map(id => findItemById(data, id))
                                .filter(Boolean) as TreeDataItem[];
                            callbacksRef.current.onSelectItemsChange(selectedItems);
                        }
                    }, 0);
                    return newSet;
                });
            },
            [multiple, onSelectChange, onSelectItemsChange, data]
        );

        const expandedItemIds = React.useMemo(() => {
            if (!initialSelectedItemIds || initialSelectedItemIds.length === 0) {
                return [] as string[]
            }

            const ids: string[] = []

            function walkTreeItems(
                items: TreeDataItem[] | TreeDataItem,
                targetIds: string[]
            ) {
                if (Array.isArray(items)) {
                    for (let i = 0; i < items.length; i++) {
                        ids.push(items[i]!.id)
                        if (walkTreeItems(items[i]!, targetIds) && !expandAll) {
                            return true
                        }
                        if (!expandAll) ids.pop()
                    }
                } else if (!expandAll && targetIds.includes(items.id)) {
                    return true
                } else if (items.children) {
                    return walkTreeItems(items.children, targetIds)
                }
            }

            walkTreeItems(data, initialSelectedItemIds)
            return ids
        }, [data, expandAll, initialSelectedItemIds])

        return (
            <div className={cn('overflow-hidden relative p-2', className)}>
                <TreeItem
                    data={data}
                    ref={ref}
                    clickRowToSelect={clickRowToSelect}
                    selectedItemIds={selectedItemIds}
                    handleSelectChange={handleSelectChange}
                    expandedItemIds={expandedItemIds}
                    defaultLeafIcon={defaultLeafIcon}
                    defaultNodeIcon={defaultNodeIcon}
                    {...props}
                />
            </div>
        )
    }
)
TreeView.displayName = 'TreeView'

// Helper function to find item by ID in tree
function findItemById(items: TreeDataItem[] | TreeDataItem, id: string): TreeDataItem | undefined {
    if (Array.isArray(items)) {
        for (const item of items) {
            const found = findItemById(item, id);
            if (found) return found;
        }
    } else {
        if (items.id === id) return items;
        if (items.children) {
            return findItemById(items.children, id);
        }
    }
    return undefined;
}

type TreeItemProps = TreeProps & {
    selectedItemIds: Set<string>
    handleSelectChange: (item: TreeDataItem | undefined) => void
    expandedItemIds: string[]
    clickRowToSelect?: boolean
    defaultNodeIcon?: React.ComponentType<{ className?: string }>
    defaultLeafIcon?: React.ComponentType<{ className?: string }>
}

const TreeItem = React.forwardRef<HTMLDivElement, TreeItemProps>(
    (
        {
            className,
            data,
            selectedItemIds,
            handleSelectChange,
            clickRowToSelect,
            expandedItemIds,
            defaultNodeIcon,
            defaultLeafIcon,
            ...props
        },
        ref
    ) => {
        if (!Array.isArray(data)) {
            data = [data]
        }

        return (
            <div ref={ref} role="tree" className={className} {...props}>
                <ul>
                    {data.map((item) => (
                        <li key={item.id}>
                            {item.children ? (
                                <TreeNode
                                    item={item}
                                    selectedItemIds={selectedItemIds}
                                    expandedItemIds={expandedItemIds}
                                    clickRowToSelect={clickRowToSelect}
                                    handleSelectChange={handleSelectChange}
                                    defaultNodeIcon={defaultNodeIcon}
                                    defaultLeafIcon={defaultLeafIcon}
                                />
                            ) : (
                                <TreeLeaf
                                    item={item}
                                    clickRowToSelect={clickRowToSelect}
                                    selectedItemIds={selectedItemIds}
                                    handleSelectChange={handleSelectChange}
                                    defaultLeafIcon={defaultLeafIcon}
                                />
                            )}
                        </li>
                    ))}
                </ul>
            </div>
        )
    }
)
TreeItem.displayName = 'TreeItem'

interface TreeNodeProps {
    item: TreeDataItem
    handleSelectChange: (item: TreeDataItem | undefined) => void
    expandedItemIds: string[]
    clickRowToSelect?: boolean
    selectedItemIds: Set<string>
    defaultNodeIcon?: React.ComponentType<{ className?: string }>
    defaultLeafIcon?: React.ComponentType<{ className?: string }>
}

const TreeNode = ({
    item,
    handleSelectChange,
    expandedItemIds,
    clickRowToSelect,
    selectedItemIds,
    defaultNodeIcon,
    defaultLeafIcon
}: TreeNodeProps) => {
    const [value, setValue] = React.useState(
        expandedItemIds.includes(item.id) ? [item.id] : []
    )

    // const hasSelectedChildren = React.useMemo(() => {
    //     if (!item.children) return false;
    //     return item.children.some(child =>
    //         selectedItemIds.has(child.id) ||
    //         (child.children && hasSelectedChildrenRecursive(child, selectedItemIds)));
    // }, [item.children, selectedItemIds]);

    const isSelected = selectedItemIds.has(item.id);

    return (
        <AccordionPrimitive.Root
            type="multiple"
            value={value}
            onValueChange={(s) => setValue(s)}
        >
            <AccordionPrimitive.Item value={item.id}>
                <AccordionTrigger
                    className={cn(
                        "flex items-center w-full py-2",
                        treeVariants(),
                        isSelected && selectedTreeVariants()
                    )}
                    onClick={(e) => {
                        e.stopPropagation();
                        if (clickRowToSelect) {
                            handleSelectChange(item);
                        }
                        item.onClick?.();
                    }}
                >
                    <div className="flex items-center min-w-0 flex-shrink-0">
                        <TreeIcon
                            item={item}
                            isSelected={isSelected}
                            isOpen={value.includes(item.id)}
                            default={defaultNodeIcon}
                            onCheck={() => { handleSelectChange(item) }}
                        />
                        <span className="ml-2 text-[13px] truncate">
                            {item.name}
                        </span>
                    </div>

                    {item.attributes && (
                        <span className="mx-auto text-[13px] text-muted-foreground whitespace-nowrap">
                            {item.attributes}
                        </span>
                    )}

                    {item.badge && (
                        <TreeBadge isSelected={isSelected}>
                            {item.badge}
                        </TreeBadge>
                    )}
                </AccordionTrigger>
                <AccordionContent className="ml-4 pl-1 border-l">
                    <TreeItem
                        data={item.children ? item.children : item}
                        selectedItemIds={selectedItemIds}
                        clickRowToSelect={clickRowToSelect}
                        handleSelectChange={handleSelectChange}
                        expandedItemIds={expandedItemIds}
                        defaultLeafIcon={defaultLeafIcon}
                        defaultNodeIcon={defaultNodeIcon}
                    />
                </AccordionContent>
            </AccordionPrimitive.Item>
        </AccordionPrimitive.Root>
    )
}

// function hasSelectedChildrenRecursive(item: TreeDataItem, selectedItemIds: Set<string>): boolean {
//     if (!item.children) return false;
//     return item.children.some(child =>
//         selectedItemIds.has(child.id) ||
//         (child.children && hasSelectedChildrenRecursive(child, selectedItemIds)));
// }

interface TreeLeafProps extends React.HTMLAttributes<HTMLDivElement> {
    item: TreeDataItem
    selectedItemIds: Set<string>
    clickRowToSelect?: boolean
    handleSelectChange: (item: TreeDataItem | undefined) => void
    defaultLeafIcon?: React.ComponentType<{ className?: string }>
}

const TreeLeaf = React.forwardRef<HTMLDivElement, TreeLeafProps>(
    (
        {
            className,
            item,
            clickRowToSelect,
            selectedItemIds,
            handleSelectChange,
            defaultLeafIcon,
            ...props
        },
        ref
    ) => {
        return (
            <div
                ref={ref}
                className={cn(
                    "ml-5 flex items-center py-2 cursor-pointer before:right-1",
                    treeVariants(),
                    className,
                    selectedItemIds.has(item.id) && selectedTreeVariants()
                )}
                onClick={(e) => {
                    e.stopPropagation();
                    if (clickRowToSelect) {
                        handleSelectChange(item);
                    }
                    item.onClick?.();
                }}
                {...props}
            >
                <div className="flex items-center min-w-0 flex-shrink-0">
                    <TreeIcon
                        item={item}
                        isSelected={selectedItemIds.has(item.id)}
                        default={defaultLeafIcon}
                        onCheck={() => { handleSelectChange(item) }}
                    />
                    <span className="ml-2 text-[13px] truncate">
                        {item.name}
                    </span>
                </div>

                {item.attributes && (
                    <span className="mx-auto text-[13px] text-muted-foreground whitespace-nowrap">
                        {item.attributes}
                    </span>
                )}

                {item.badge && (
                    <TreeBadge isSelected={selectedItemIds.has(item.id)}>
                        {item.badge}
                    </TreeBadge>
                )}
            </div>
        )
    }
)
TreeLeaf.displayName = 'TreeLeaf'

const AccordionTrigger = React.forwardRef<
    React.ElementRef<typeof AccordionPrimitive.Trigger>,
    React.ComponentPropsWithoutRef<typeof AccordionPrimitive.Trigger>
>(({ className, children, ...props }, ref) => (
    <AccordionPrimitive.Header>
        <AccordionPrimitive.Trigger
            ref={ref}
            className={cn(
                'flex flex-1 w-full items-center py-2 transition-all first:[&[data-state=open]>svg]:rotate-90',
                className
            )}
            {...props}
            onClick={(e) => {
                e.stopPropagation()
                if (props.onClick) {
                    props.onClick(e)
                }
            }}
        >
            <ChevronRight className="h-4 w-4 shrink-0 transition-transform duration-200 text-accent-foreground/50 mr-1" />
            {children}
        </AccordionPrimitive.Trigger>
    </AccordionPrimitive.Header>
))
AccordionTrigger.displayName = AccordionPrimitive.Trigger.displayName

const AccordionContent = React.forwardRef<
    React.ElementRef<typeof AccordionPrimitive.Content>,
    React.ComponentPropsWithoutRef<typeof AccordionPrimitive.Content>
>(({ className, children, ...props }, ref) => (
    <AccordionPrimitive.Content
        ref={ref}
        className={cn(
            'overflow-hidden text-[13px] transition-all data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down',
            className
        )}
        {...props}
    >
        <div className="pb-1 pt-0">{children}</div>
    </AccordionPrimitive.Content>
))
AccordionContent.displayName = AccordionPrimitive.Content.displayName

interface TreeIconProps {
    item: TreeDataItem;
    isOpen?: boolean;
    isSelected?: boolean;
    default?: React.ComponentType<{ className?: string }>;
    onCheck?: (checked: boolean) => void;
}

const TreeIcon = ({
    item,
    isOpen,
    isSelected,
    default: defaultIcon,
    onCheck,
}: TreeIconProps) => {
    let Icon = defaultIcon;
    if (isOpen && item.openIcon) {
        Icon = item.openIcon;
    } else if (item.icon) {
        Icon = item.icon;
    }

    return (
        <div className="flex items-center gap-2">
            <input
                type="checkbox"
                checked={isSelected}
                onChange={() => onCheck?.(!isSelected)}
                onClick={(e) => e.stopPropagation()}
                className={cn(
                    "h-4 w-4 rounded border border-primary dark:border-white shadow transition-all duration-200",
                    "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
                    isSelected
                        ? "bg-black dark:bg-white text-primary-foreground"
                        : "bg-transparent",
                    "appearance-none cursor-pointer flex items-center justify-center relative",
                    "after:content-[''] after:w-1.5 after:h-2",
                    "after:border-r-2 after:border-b-2 after:rotate-45 after:mt-[-2px]",
                    isSelected
                        ? "after:block after:border-white dark:after:border-black after:z-10"
                        : "after:hidden"
                )}
            />
            {Icon && <Icon className="h-4 w-4 shrink-0" />}
        </div>
    );
};

interface TreeBadgeProps {
    children: React.ReactNode
    isSelected: boolean
    showOnSelectedOnly?: boolean
}

const TreeBadge = ({
    children,
    isSelected,
    showOnSelectedOnly = false
}: TreeBadgeProps) => {
    return (
        <div
            className={cn(
                showOnSelectedOnly
                    ? isSelected
                        ? 'block'
                        : 'hidden'
                    : 'block',
                'absolute right-3 group-hover:block'
            )}
        >
            {children}
        </div>
    )
}

export { TreeView, type TreeDataItem }