/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { ThemeSwitch } from "../theme-switch";
import { ProfileDropdown } from "../profile-dropdown";
import { Header } from "./header";
import { NotificationPopover } from "./notification";
import { GithubLinkButton } from "./github";

export const FixedHeader = () => {
    return (
        <Header fixed>
            {/* <Search /> */}
            <div className='ml-auto flex items-center space-x-4'>
                <NotificationPopover />
                <GithubLinkButton />
                <ThemeSwitch />
                <ProfileDropdown />
            </div>
        </Header>
    );
};