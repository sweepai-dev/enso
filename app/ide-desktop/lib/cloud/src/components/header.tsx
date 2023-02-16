import * as React from "react";

export const Header: React.FC<any> = () => {
    return (
        <header className="bg-white">
            <div className="mx-auto py-6 px-4 sm:px-6 lg:px-8">
                <h1 className="text-3xl font-bold tracking-tight text-gray-900">
                    Drive
                </h1>
            </div>
        </header>
    );
};