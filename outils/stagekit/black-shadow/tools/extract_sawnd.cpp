#include "lavaResawndz.h"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
    if (argc < 3) {
        std::cerr << "Usage: extract_sawnd <input.sawnd> <output_dir>\n";
        return 2;
    }
    const std::string input = argv[1];
    const std::filesystem::path output = argv[2];
    std::filesystem::create_directories(output);

    auto banks = lava::brawl::parseSawndFie(input);
    if (banks.empty()) {
        std::cerr << "No valid SAWND bank found: " << input << "\n";
        return 3;
    }

    std::size_t extracted = 0;
    for (const auto& bank : banks) {
        std::cout << "BANK group=0x" << std::hex << bank.groupID << std::dec
                  << " entries=" << bank.fileEntries.size() << "\n";
        for (const auto& entry : bank.fileEntries) {
            lava::brawl::brsarFileFileContents file;
            file.fileID = entry.fileID;
            file.header = entry.headerContent;
            file.data = entry.dataContent;

            lava::brawl::rwsd rwsd;
            if (!rwsd.populate(file)) {
                std::cout << "SKIP file=" << entry.fileID << " (not a readable RWSD)\n";
                continue;
            }

            // correspondance son (entrée DATA du RWSD) -> onde, pour retrouver quel son est quel fichier
            for (std::size_t di = 0; di < rwsd.dataSection.entries.size(); ++di) {
                std::cout << "DATA group=" << bank.groupID << " file=" << entry.fileID << " data=" << di
                          << " wave=" << rwsd.dataSection.entries[di].ntWaveIndex << "\n";
            }
            for (std::size_t wave = 0; wave < rwsd.waveSection.entries.size(); ++wave) {
                const auto name = "group_" + std::to_string(bank.groupID)
                    + "_file_" + std::to_string(entry.fileID)
                    + "_wave_" + std::to_string(wave) + ".dsp";
                const auto path = (output / name).string();
                auto dsp = rwsd.exportWaveRawDataToDSP(static_cast<unsigned long>(wave));
                std::ofstream stream(path, std::ios::binary);
                if (!dsp.body.empty() && stream.is_open() && dsp.exportContents(stream)) {
                    stream.close();
                    std::error_code ec;
                    const auto size = std::filesystem::file_size(path, ec);
                    std::cout << "DSP_EXTRACTED " << name << " bytes="
                              << (ec ? 0 : size) << "\n";
                    ++extracted;
                } else {
                    std::cout << "FAILED " << name << "\n";
                }
            }
        }
    }
    std::cout << "TOTAL_DSP_EXTRACTED " << extracted << "\n";
    return extracted ? 0 : 4;
}
