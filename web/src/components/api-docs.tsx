/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Card } from "@/components/ui/card";
import { FixedHeader } from "./layout/fixed-header";
import { Main } from "./layout/main";
import Logo from '@/assets/logo.svg'

const docsOptions = [
  { name: "Swagger UI", path: "/api-docs/swagger" },
  { name: "ReDoc", path: "/api-docs/redoc" },
  { name: "OpenAPI Explorer", path: "/api-docs/explorer" },
  { name: "Scalar", path: "/api-docs/scalar" },
  { name: "Download Spec YAML", path: "/api-docs/spec.yaml" }
];

export default function APIDocs() {
  const handleCardClick = (path: string) => {
    // Open in new tab
    window.open(path, '_blank', 'noopener,noreferrer');
  };

  return (
    <>

      <FixedHeader />
      <Main>
        <div className='mb-2 flex items-center justify-between space-y-2 flex-wrap gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>API Documentation</h2>
            <p className='text-muted-foreground'>
              Choose your preferred API documentation type
            </p>
          </div>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 flex-row lg:space-x-12 space-y-0'>
          <div className='m-auto flex h-full w-full flex-col items-center justify-center gap-6 p-4'>
            <div className="grid w-full gap-4 sm:grid-cols-1 md:grid-cols-2 xl:max-w-4xl">
              {docsOptions.map((option) => (
                <Card
                  key={option.name}
                  className="cursor-pointer p-6 transition-all hover:bg-accent hover:text-accent-foreground hover:shadow-md"
                  onClick={() => handleCardClick(option.path)}
                >
                  <div className="flex items-center gap-4">
                    <img
                      src={Logo}
                      className="max-h-[66px] w-auto opacity-20 saturate-0 transition-all duration-300 hover:opacity-100 hover:saturate-100 object-contain"
                      alt="RustMailer Logo"
                    />
                    <h3 className="text-sm font-medium">{option.name}</h3>
                  </div>
                </Card>
              ))}
            </div>
          </div>
        </div>
      </Main>
    </>
  );
}