/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */


import * as React from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Blend, Minus, Plus, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/hooks/use-toast";
import { validateFlag } from "@/lib/utils";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useFlagMessageMutation } from "@/hooks/use-flag-messages";
import { MailboxData } from "@/api/mailbox/api";

type EmailFlag = "Seen" | "Answered" | "Flagged" | "Deleted" | "Draft" | "Recent" | "MayCreate" | "Custom";
type EnvelopeFlag = {
    flag: EmailFlag;
    custom: string | null;
};

const PREDEFINED_FLAGS: EmailFlag[] = ["Seen", "Answered", "Flagged", "Deleted", "Draft", "Recent", "MayCreate"];

interface Props {
    selectedAccountId?: number;
    selectedMailbox?: MailboxData;
    selectedUids: number[];
}

export function CustomFlagInput({ selectedAccountId, selectedMailbox, selectedUids }: Props) {
    const { mutate: flagMessage, isPending } = useFlagMessageMutation();
    const [flagName, setFlagName] = React.useState("");
    const [selectedPredefinedFlag, setSelectedPredefinedFlag] = React.useState<EmailFlag | null>(null);
    const [flags, setFlags] = React.useState<EnvelopeFlag[]>([]);
    const [action, setAction] = React.useState<"add" | "remove" | "overwrite">("add");

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setFlagName(e.target.value);
    };

    const handleAddFlag = () => {
        if (selectedPredefinedFlag && selectedPredefinedFlag !== "Custom") {
            const newFlag: EnvelopeFlag = {
                flag: selectedPredefinedFlag,
                custom: null,
            };

            if (flags.some((f) => f.flag === newFlag.flag && f.custom === newFlag.custom)) {
                toast({
                    title: "Error",
                    description: `"${newFlag.flag}" flag already exists.`,
                    variant: "destructive",
                });
                return;
            }

            setFlags((prev) => [...prev, newFlag]);
            setSelectedPredefinedFlag(null); // Reset for predefined flags
            return;
        }

        if (selectedPredefinedFlag === "Custom") {
            const validationError = validateFlag(flagName);
            if (validationError) {
                toast({
                    title: "Error",
                    description: validationError,
                    variant: "destructive",
                });
                return;
            }

            const trimmedFlag = flagName.trim();
            if (flags.some((f) => f.flag === "Custom" && f.custom === trimmedFlag)) {
                toast({
                    title: "Error",
                    description: `"${trimmedFlag}" custom flag already exists.`,
                    variant: "destructive",
                });
                return;
            }

            const newFlag: EnvelopeFlag = {
                flag: "Custom",
                custom: trimmedFlag,
            };

            setFlags((prev) => [...prev, newFlag]);
            setFlagName(""); // Clear input but keep "Custom" selected
            // Do NOT reset selectedPredefinedFlag to null, so input stays visible
        }
    };

    const handleRemoveFlag = (flagToRemove: EnvelopeFlag) => {
        setFlags((prevFlags) =>
            prevFlags.filter((flag) => flag.flag !== flagToRemove.flag || flag.custom !== flagToRemove.custom)
        );
    };

    const onSubmit = () => {
        if (selectedAccountId && selectedMailbox && flags.length > 0) {
            const actionPayload: {
                uids: number[];
                mailbox: string;
                action?: {
                    add?: EnvelopeFlag[];
                    remove?: EnvelopeFlag[];
                    overwrite?: EnvelopeFlag[];
                };
            } = {
                uids: selectedUids,
                mailbox: selectedMailbox?.name,
            };

            actionPayload.action = {
                [action]: flags,
            } as { add?: EnvelopeFlag[]; remove?: EnvelopeFlag[]; overwrite?: EnvelopeFlag[] };

            flagMessage({ accountId: selectedAccountId, payload: actionPayload });
        }
    };

    const formatFlagDisplay = (flag: EnvelopeFlag) => {
        return flag.flag === "Custom" ? flag.custom : flag.flag;
    };

    const actionButtonText = React.useMemo(() => {
        switch (action) {
            case "add":
                return "Add Flags";
            case "remove":
                return "Remove Flags";
            case "overwrite":
                return "Overwrite Flags";
            default:
                return "Submit";
        }
    }, [action]);

    const actionButtonIcon = React.useMemo(() => {
        switch (action) {
            case "add":
                return <Plus className="h-4 w-4 mr-2" />;
            case "remove":
                return <Minus className="h-4 w-4 mr-2" />;
            case "overwrite":
                return <Blend className="h-4 w-4 mr-2" />;
            default:
                return null;
        }
    }, [action]);

    const stageButtonText = React.useMemo(() => {
        switch (action) {
            case "add":
                return "Stage for Addition";
            case "remove":
                return "Stage for Removal";
            case "overwrite":
                return "Stage for Overwrite";
            default:
                return "Stage";
        }
    }, [action]);

    return (
        <div>
            <div className="mb-2">
                <Select value={action} onValueChange={(value) => setAction(value as "add" | "remove" | "overwrite")}>
                    <SelectTrigger className="flex items-center justify-center text-xs">
                        <SelectValue placeholder="Select an action" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="add">Add</SelectItem>
                        <SelectItem value="remove">Remove</SelectItem>
                        <SelectItem value="overwrite">Overwrite</SelectItem>
                    </SelectContent>
                </Select>
            </div>

            <div className="flex items-center justify-between space-x-2 mb-2">
                <Select
                    value={selectedPredefinedFlag || undefined}
                    onValueChange={(value) => setSelectedPredefinedFlag(value as EmailFlag)}
                >
                    <SelectTrigger className="flex-1">
                        <SelectValue placeholder="Select flag type" />
                    </SelectTrigger>
                    <SelectContent>
                        {PREDEFINED_FLAGS.map((flag) => (
                            <SelectItem key={flag} value={flag}>
                                {flag}
                            </SelectItem>
                        ))}
                        <SelectItem value="Custom">Custom</SelectItem>
                    </SelectContent>
                </Select>
                {selectedPredefinedFlag && selectedPredefinedFlag !== "Custom" && (
                    <Button
                        variant="outline"
                        size="sm"
                        className="text-xs px-2 py-1"
                        onClick={handleAddFlag}
                    >
                        <Plus className="h-3 w-3 mr-1" />
                        {stageButtonText}
                    </Button>
                )}
            </div>

            {selectedPredefinedFlag === "Custom" && (
                <div className="flex items-center justify-between space-x-2 mb-4">
                    <Input
                        placeholder="Enter custom flag name"
                        value={flagName}
                        onChange={handleInputChange}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") {
                                handleAddFlag();
                            }
                        }}
                        className="flex-1"
                    />
                    <Button
                        variant="outline"
                        size="sm"
                        className="text-xs px-2 py-1"
                        onClick={handleAddFlag}
                    >
                        <Plus className="h-3 w-3 mr-1" />
                        {stageButtonText}
                    </Button>
                </div>
            )}

            <div className="mt-2">
                {flags.length > 0 ? (
                    <div className="flex flex-col gap-2">
                        <div className="flex flex-wrap gap-2">
                            {flags.map((flag, index) => (
                                <Badge key={index} variant="outline" className="text-sm flex items-center gap-1">
                                    {formatFlagDisplay(flag)}
                                    <button
                                        type="button"
                                        onClick={() => handleRemoveFlag(flag)}
                                        className="rounded-full p-0.5 hover:bg-gray-200"
                                    >
                                        <X className="h-3 w-3" />
                                    </button>
                                </Badge>
                            ))}
                        </div>
                        <Button
                            className="w-full flex items-center justify-center text-xs"
                            onClick={onSubmit}
                            disabled={isPending || flags.length === 0}
                        >
                            {actionButtonIcon}
                            {actionButtonText}
                        </Button>
                    </div>
                ) : (
                    <p className="text-sm text-gray-500">No flags added yet.</p>
                )}
            </div>
        </div>
    );
}