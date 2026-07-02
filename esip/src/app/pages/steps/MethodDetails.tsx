import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Input } from '../../components/ui/input';
import { Label } from '../../components/ui/label';
import { Card } from '../../components/ui/card';
import { ArrowRight, ArrowLeft, AlertCircle } from 'lucide-react';

export function MethodDetails() {
  const navigate = useNavigate();
  const { screens, selectedMethods, updateScreen, setCurrentStep } = useEstimationStore();
  
  // Set current step when component mounts
  useEffect(() => {
    setCurrentStep(4);
  }, [setCurrentStep]);
  
  // Track validation errors
  const [pertErrors, setPertErrors] = useState<Record<string, string>>({});
  const [fpaErrors, setFpaErrors] = useState<Record<string, string>>({});

  // Helper: Check if a screen has children
  const hasChildren = (screenId: string): boolean => {
    return screens.some(s => s.parentId === screenId);
  };

  // Helper: Get all children of a screen
  const getChildren = (screenId: string) => {
    return screens.filter(s => s.parentId === screenId);
  };

  // Helper: Calculate rolled-up value for parent screens
  const calculateRollup = (screenId: string, method: string): number => {
    const children = getChildren(screenId);
    if (children.length === 0) return 0;

    let total = 0;
    children.forEach(child => {
      // If child also has children, recursively calculate
      if (hasChildren(child.id)) {
        total += calculateRollup(child.id, method);
      } else {
        // Leaf node - get its value
        switch (method) {
          case 'PERT':
            if (child.pertData) {
              total += (child.pertData.optimistic + 4 * child.pertData.mostLikely + child.pertData.pessimistic) / 6;
            }
            break;
          case 'STORY_POINTS':
            total += child.storyPoints || 0;
            break;
          case 'TSHIRT':
            total += child.tshirtSize || 0;
            break;
          case 'FPA':
            if (child.fpaData) {
              const unadjustedFP = 
                (child.fpaData.externalInputs || 0) * 4 +
                (child.fpaData.externalOutputs || 0) * 5 +
                (child.fpaData.externalInquiries || 0) * 4 +
                (child.fpaData.internalLogicalFiles || 0) * 10 +
                (child.fpaData.externalInterfaceFiles || 0) * 7;
              total += unadjustedFP;
            }
            break;
          case 'UCP':
            if (child.ucpData) {
              const uucp = 
                (child.ucpData.simpleActors || 0) * 1 +
                (child.ucpData.averageActors || 0) * 2 +
                (child.ucpData.complexActors || 0) * 3 +
                (child.ucpData.simpleUseCases || 0) * 5 +
                (child.ucpData.averageUseCases || 0) * 10 +
                (child.ucpData.complexUseCases || 0) * 15;
              total += uucp;
            }
            break;
        }
      }
    });
    return total;
  };

  // Validate PERT data: O ≤ M ≤ P
  const validatePert = (screenId: string, optimistic: number, mostLikely: number, pessimistic: number) => {
    const errors: string[] = [];
    
    if (optimistic > mostLikely) {
      errors.push('Optimistic must be ≤ Most Likely');
    }
    if (mostLikely > pessimistic) {
      errors.push('Most Likely must be ≤ Pessimistic');
    }
    if (optimistic > pessimistic) {
      errors.push('Optimistic must be ≤ Pessimistic');
    }
    
    setPertErrors(prev => ({
      ...prev,
      [screenId]: errors.join(', ')
    }));
    
    return errors.length === 0;
  };

  // Check if form is complete (required fields filled)
  const isFormComplete = () => {
    // Check if any method has incomplete data (only for leaf items)
    for (const screen of screens) {
      // Skip parent screens - they auto-calculate from children
      if (hasChildren(screen.id)) {
        continue;
      }

      if (selectedMethods.includes('PERT')) {
        if (!screen.pertData?.optimistic || !screen.pertData?.mostLikely || !screen.pertData?.pessimistic) {
          return false;
        }
        // Also check for validation errors
        if (pertErrors[screen.id]) {
          return false;
        }
      }
      if (selectedMethods.includes('STORY_POINTS')) {
        if (!screen.storyPoints) {
          return false;
        }
      }
      if (selectedMethods.includes('TSHIRT')) {
        if (!screen.tshirtSize) {
          return false;
        }
      }
      if (selectedMethods.includes('FPA')) {
        // FPA requires at least one function point component filled
        if (!screen.fpaData || 
            (screen.fpaData.externalInputs === 0 && 
             screen.fpaData.externalOutputs === 0 && 
             screen.fpaData.externalInquiries === 0 && 
             screen.fpaData.internalLogicalFiles === 0 && 
             screen.fpaData.externalInterfaceFiles === 0)) {
          return false;
        }
      }
      if (selectedMethods.includes('UCP')) {
        // UCP requires at least one actor or use case filled
        if (!screen.ucpData || 
            (screen.ucpData.simpleActors === 0 && 
             screen.ucpData.averageActors === 0 && 
             screen.ucpData.complexActors === 0 && 
             screen.ucpData.simpleUseCases === 0 && 
             screen.ucpData.averageUseCases === 0 && 
             screen.ucpData.complexUseCases === 0)) {
          return false;
        }
      }
    }
    return true;
  };

  const handleNext = () => {
    if (isFormComplete()) {
      setCurrentStep(5);
      navigate('/wizard/step/5');
    } else {
      alert('Please fill in all required fields and correct any validation errors.');
    }
  };

  const handleBack = () => {
    setCurrentStep(3);
    navigate('/wizard/step/3');
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Method-Specific Details</h2>
        <p className="text-gray-600">
          Provide estimation data for each screen based on selected method(s)
        </p>
      </div>

      {/* PERT Method */}
      {selectedMethods.includes('PERT') && (
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-blue-900">📊 PERT (Three-Point Estimation)</h3>
          
          {screens.map((screen) => {
            const isParent = hasChildren(screen.id);
            const childrenCount = getChildren(screen.id).length;
            const rollupValue = isParent ? calculateRollup(screen.id, 'PERT') : 0;

            return (
              <Card key={screen.id} className={`p-6 ${isParent ? 'bg-blue-50 border-2 border-blue-300' : ''}`}>
                <div className="flex items-center justify-between mb-4">
                  <h4 className="font-semibold text-gray-900">{screen.name}</h4>
                  {isParent && (
                    <span className="text-xs bg-blue-600 text-white px-3 py-1 rounded-full font-semibold">
                      📁 Parent ({childrenCount} children)
                    </span>
                  )}
                </div>
                
                {isParent ? (
                  // Parent: Show auto-calculated rollup
                  <div className="bg-gradient-to-r from-blue-100 to-blue-50 border border-blue-300 rounded-lg p-4">
                    <div className="flex items-center gap-3 mb-2">
                      <span className="text-2xl">📊</span>
                      <div>
                        <div className="text-sm font-semibold text-blue-900">Auto-Calculated (Sum of Children)</div>
                        <div className="text-xs text-blue-700">This is a container - estimates come from child items</div>
                      </div>
                    </div>
                    <div className="mt-3 pt-3 border-t border-blue-300">
                      <span className="text-sm font-semibold text-blue-900">Total Expected Time: </span>
                      <span className="text-2xl font-bold text-blue-700">{rollupValue.toFixed(1)} hours</span>
                    </div>
                  </div>
                ) : (
                  // Leaf: Show input fields
                  <>
                    <div className="grid md:grid-cols-3 gap-4">
                      <div>
                        <Label>Optimistic (hours)</Label>
                        <Input
                          type="number"
                          min="1"
                          value={screen.pertData?.optimistic || ''}
                          onChange={(e) => {
                            const optimistic = Number(e.target.value);
                            const mostLikely = screen.pertData?.mostLikely || 0;
                            const pessimistic = screen.pertData?.pessimistic || 0;
                            
                            updateScreen(screen.id, {
                              pertData: {
                                optimistic: optimistic,
                                mostLikely: mostLikely,
                                pessimistic: pessimistic,
                              },
                            });
                            
                            validatePert(screen.id, optimistic, mostLikely, pessimistic);
                          }}
                          className="mt-1"
                          placeholder="Best case"
                        />
                      </div>
                      
                      <div>
                        <Label>Most Likely (hours)</Label>
                        <Input
                          type="number"
                          min="1"
                          value={screen.pertData?.mostLikely || ''}
                          onChange={(e) => {
                            const mostLikely = Number(e.target.value);
                            const optimistic = screen.pertData?.optimistic || 0;
                            const pessimistic = screen.pertData?.pessimistic || 0;
                            
                            updateScreen(screen.id, {
                              pertData: {
                                optimistic: optimistic,
                                mostLikely: mostLikely,
                                pessimistic: pessimistic,
                              },
                            });
                            
                            validatePert(screen.id, optimistic, mostLikely, pessimistic);
                          }}
                          className="mt-1"
                          placeholder="Realistic"
                        />
                      </div>
                      
                      <div>
                        <Label>Pessimistic (hours)</Label>
                        <Input
                          type="number"
                          min="1"
                          value={screen.pertData?.pessimistic || ''}
                          onChange={(e) => {
                            const pessimistic = Number(e.target.value);
                            const optimistic = screen.pertData?.optimistic || 0;
                            const mostLikely = screen.pertData?.mostLikely || 0;
                            
                            updateScreen(screen.id, {
                              pertData: {
                                optimistic: optimistic,
                                mostLikely: mostLikely,
                                pessimistic: pessimistic,
                              },
                            });
                            
                            validatePert(screen.id, optimistic, mostLikely, pessimistic);
                          }}
                          className="mt-1"
                          placeholder="Worst case"
                        />
                      </div>
                    </div>
                    
                    {/* Validation Errors */}
                    {pertErrors[screen.id] && (
                      <div className="mt-3 bg-red-50 border border-red-200 rounded p-3 flex items-start gap-2">
                        <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                        <span className="text-sm text-red-700">{pertErrors[screen.id]}</span>
                      </div>
                    )}
                    
                    {screen.pertData?.optimistic && screen.pertData?.mostLikely && screen.pertData?.pessimistic && !pertErrors[screen.id] && (
                      <div className="mt-3 bg-blue-50 border border-blue-200 rounded p-3">
                        <span className="text-sm font-semibold text-blue-900">Expected Time: </span>
                        <span className="text-lg font-bold text-blue-700">
                          {((screen.pertData.optimistic + 4 * screen.pertData.mostLikely + screen.pertData.pessimistic) / 6).toFixed(1)} hours
                        </span>
                      </div>
                    )}
                  </>
                )}
              </Card>
            );
          })}
        </div>
      )}

      {/* Story Points */}
      {selectedMethods.includes('STORY_POINTS') && (
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-purple-900">🎯 Story Points</h3>
          
          {screens.map((screen) => (
            <Card key={screen.id} className="p-6">
              <h4 className="font-semibold text-gray-900 mb-4">{screen.name}</h4>
              
              <div>
                <Label>Story Points (Fibonacci)</Label>
                <select
                  className="w-full mt-1 p-2 border rounded"
                  defaultValue={screen.storyPoints || 5}
                  onChange={(e) =>
                    updateScreen(screen.id, { storyPoints: Number(e.target.value) })
                  }
                >
                  {[1, 2, 3, 5, 8, 13, 21, 34].map((points) => (
                    <option key={points} value={points}>
                      {points} points
                    </option>
                  ))}
                </select>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* T-Shirt Sizes */}
      {selectedMethods.includes('TSHIRT') && (
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-green-900">👕 T-Shirt Sizes</h3>
          
          {screens.map((screen) => (
            <Card key={screen.id} className="p-6">
              <h4 className="font-semibold text-gray-900 mb-4">{screen.name}</h4>
              
              <div>
                <Label>T-Shirt Size</Label>
                <select
                  className="w-full mt-1 p-2 border rounded"
                  defaultValue={screen.tshirtSize || 'S'}
                  onChange={(e) =>
                    updateScreen(screen.id, { tshirtSize: e.target.value })
                  }
                >
                  <option value="S">Small (S)</option>
                  <option value="M">Medium (M)</option>
                  <option value="L">Large (L)</option>
                  <option value="XL">Extra Large (XL)</option>
                </select>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Function Point Analysis (FPA) */}
      {selectedMethods.includes('FPA') && (
        <div className="space-y-4">
          <div>
            <h3 className="text-xl font-semibold text-indigo-900 mb-2">📐 Function Point Analysis (FPA)</h3>
            <p className="text-sm text-gray-600 bg-indigo-50 border border-indigo-200 rounded p-3">
              Count the number of each type of function for this screen/feature. FPA uses industry-standard weights to calculate effort.
            </p>
          </div>
          
          {screens.map((screen) => {
            const fpaData = screen.fpaData || {
              externalInputs: 0,
              externalOutputs: 0,
              externalInquiries: 0,
              internalLogicalFiles: 0,
              externalInterfaceFiles: 0
            };
            
            // Calculate function points in real-time
            const functionPoints = 
              (fpaData.externalInputs * 4) + 
              (fpaData.externalOutputs * 5) + 
              (fpaData.externalInquiries * 4) + 
              (fpaData.internalLogicalFiles * 10) + 
              (fpaData.externalInterfaceFiles * 7);
            const estimatedHours = functionPoints * 8;

            return (
              <Card key={screen.id} className="p-6">
                <h4 className="font-semibold text-gray-900 mb-4">{screen.name}</h4>
                
                <div className="grid md:grid-cols-2 gap-4 mb-4">
                  <div>
                    <Label className="flex items-center gap-2">
                      External Inputs (EI)
                      <span className="text-xs text-gray-500 font-normal">× 4 FP</span>
                    </Label>
                    <Input
                      type="number"
                      min="0"
                      value={fpaData.externalInputs}
                      onChange={(e) => {
                        updateScreen(screen.id, {
                          fpaData: {
                            ...fpaData,
                            externalInputs: Number(e.target.value)
                          }
                        });
                      }}
                      className="mt-1"
                      placeholder="Forms, inputs"
                    />
                    <p className="text-xs text-gray-500 mt-1">User input screens, forms, data entry</p>
                  </div>

                  <div>
                    <Label className="flex items-center gap-2">
                      External Outputs (EO)
                      <span className="text-xs text-gray-500 font-normal">× 5 FP</span>
                    </Label>
                    <Input
                      type="number"
                      min="0"
                      value={fpaData.externalOutputs}
                      onChange={(e) => {
                        updateScreen(screen.id, {
                          fpaData: {
                            ...fpaData,
                            externalOutputs: Number(e.target.value)
                          }
                        });
                      }}
                      className="mt-1"
                      placeholder="Reports, exports"
                    />
                    <p className="text-xs text-gray-500 mt-1">Reports, exports, calculated outputs</p>
                  </div>

                  <div>
                    <Label className="flex items-center gap-2">
                      External Inquiries (EQ)
                      <span className="text-xs text-gray-500 font-normal">× 4 FP</span>
                    </Label>
                    <Input
                      type="number"
                      min="0"
                      value={fpaData.externalInquiries}
                      onChange={(e) => {
                        updateScreen(screen.id, {
                          fpaData: {
                            ...fpaData,
                            externalInquiries: Number(e.target.value)
                          }
                        });
                      }}
                      className="mt-1"
                      placeholder="Searches, queries"
                    />
                    <p className="text-xs text-gray-500 mt-1">Search, query, lookup functions</p>
                  </div>

                  <div>
                    <Label className="flex items-center gap-2">
                      Internal Logical Files (ILF)
                      <span className="text-xs text-gray-500 font-normal">× 10 FP</span>
                    </Label>
                    <Input
                      type="number"
                      min="0"
                      value={fpaData.internalLogicalFiles}
                      onChange={(e) => {
                        updateScreen(screen.id, {
                          fpaData: {
                            ...fpaData,
                            internalLogicalFiles: Number(e.target.value)
                          }
                        });
                      }}
                      className="mt-1"
                      placeholder="Database tables"
                    />
                    <p className="text-xs text-gray-500 mt-1">Data stored/maintained by system</p>
                  </div>

                  <div>
                    <Label className="flex items-center gap-2">
                      External Interface Files (EIF)
                      <span className="text-xs text-gray-500 font-normal">× 7 FP</span>
                    </Label>
                    <Input
                      type="number"
                      min="0"
                      value={fpaData.externalInterfaceFiles}
                      onChange={(e) => {
                        updateScreen(screen.id, {
                          fpaData: {
                            ...fpaData,
                            externalInterfaceFiles: Number(e.target.value)
                          }
                        });
                      }}
                      className="mt-1"
                      placeholder="External APIs"
                    />
                    <p className="text-xs text-gray-500 mt-1">Data referenced from external systems</p>
                  </div>
                </div>

                {functionPoints > 0 && (
                  <div className="mt-3 bg-indigo-50 border border-indigo-200 rounded p-3">
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <span className="text-sm font-semibold text-indigo-900">Function Points: </span>
                        <span className="text-lg font-bold text-indigo-700">{functionPoints} FP</span>
                      </div>
                      <div>
                        <span className="text-sm font-semibold text-indigo-900">Estimated Time: </span>
                        <span className="text-lg font-bold text-indigo-700">{estimatedHours} hours</span>
                      </div>
                    </div>
                    <p className="text-xs text-indigo-600 mt-2">Using industry standard: 8 hours per function point</p>
                  </div>
                )}
              </Card>
            );
          })}
        </div>
      )}

      {/* Use Case Points (UCP) */}
      {selectedMethods.includes('UCP') && (
        <div className="space-y-4">
          <div>
            <h3 className="text-xl font-semibold text-pink-900 mb-2">🎭 Use Case Points (UCP)</h3>
            <p className="text-sm text-gray-600 bg-pink-50 border border-pink-200 rounded p-3">
              Count actors and use cases by complexity. UCP is ideal for object-oriented and use case-driven projects.
            </p>
          </div>
          
          {screens.map((screen) => {
            const ucpData = screen.ucpData || {
              simpleActors: 0,
              averageActors: 0,
              complexActors: 0,
              simpleUseCases: 0,
              averageUseCases: 0,
              complexUseCases: 0
            };
            
            // Calculate UCP in real-time
            const uaw = (ucpData.simpleActors * 1) + (ucpData.averageActors * 2) + (ucpData.complexActors * 3);
            const uucw = (ucpData.simpleUseCases * 5) + (ucpData.averageUseCases * 10) + (ucpData.complexUseCases * 15);
            const uucp = uaw + uucw; // Unadjusted Use Case Points
            const estimatedHours = uucp * 20; // Standard: 20 hours per UCP

            return (
              <Card key={screen.id} className="p-6">
                <h4 className="font-semibold text-gray-900 mb-4">{screen.name}</h4>
                
                {/* Actors Section */}
                <div className="mb-6">
                  <h5 className="font-semibold text-pink-800 mb-3">Actors (System Interfaces)</h5>
                  <div className="grid md:grid-cols-3 gap-4">
                    <div>
                      <Label className="flex items-center gap-2">
                        Simple Actors
                        <span className="text-xs text-gray-500 font-normal">× 1</span>
                      </Label>
                      <Input
                        type="number"
                        min="0"
                        value={ucpData.simpleActors}
                        onChange={(e) => {
                          updateScreen(screen.id, {
                            ucpData: {
                              ...ucpData,
                              simpleActors: Number(e.target.value)
                            }
                          });
                        }}
                        className="mt-1"
                        placeholder="0"
                      />
                      <p className="text-xs text-gray-500 mt-1">API, defined interface</p>
                    </div>

                    <div>
                      <Label className="flex items-center gap-2">
                        Average Actors
                        <span className="text-xs text-gray-500 font-normal">× 2</span>
                      </Label>
                      <Input
                        type="number"
                        min="0"
                        value={ucpData.averageActors}
                        onChange={(e) => {
                          updateScreen(screen.id, {
                            ucpData: {
                              ...ucpData,
                              averageActors: Number(e.target.value)
                            }
                          });
                        }}
                        className="mt-1"
                        placeholder="0"
                      />
                      <p className="text-xs text-gray-500 mt-1">Interactive protocol</p>
                    </div>

                    <div>
                      <Label className="flex items-center gap-2">
                        Complex Actors
                        <span className="text-xs text-gray-500 font-normal">× 3</span>
                      </Label>
                      <Input
                        type="number"
                        min="0"
                        value={ucpData.complexActors}
                        onChange={(e) => {
                          updateScreen(screen.id, {
                            ucpData: {
                              ...ucpData,
                              complexActors: Number(e.target.value)
                            }
                          });
                        }}
                        className="mt-1"
                        placeholder="0"
                      />
                      <p className="text-xs text-gray-500 mt-1">GUI, human user</p>
                    </div>
                  </div>
                </div>

                {/* Use Cases Section */}
                <div>
                  <h5 className="font-semibold text-pink-800 mb-3">Use Cases (Functional Requirements)</h5>
                  <div className="grid md:grid-cols-3 gap-4">
                    <div>
                      <Label className="flex items-center gap-2">
                        Simple Use Cases
                        <span className="text-xs text-gray-500 font-normal">× 5</span>
                      </Label>
                      <Input
                        type="number"
                        min="0"
                        value={ucpData.simpleUseCases}
                        onChange={(e) => {
                          updateScreen(screen.id, {
                            ucpData: {
                              ...ucpData,
                              simpleUseCases: Number(e.target.value)
                            }
                          });
                        }}
                        className="mt-1"
                        placeholder="0"
                      />
                      <p className="text-xs text-gray-500 mt-1">≤3 transactions, simple logic</p>
                    </div>

                    <div>
                      <Label className="flex items-center gap-2">
                        Average Use Cases
                        <span className="text-xs text-gray-500 font-normal">× 10</span>
                      </Label>
                      <Input
                        type="number"
                        min="0"
                        value={ucpData.averageUseCases}
                        onChange={(e) => {
                          updateScreen(screen.id, {
                            ucpData: {
                              ...ucpData,
                              averageUseCases: Number(e.target.value)
                            }
                          });
                        }}
                        className="mt-1"
                        placeholder="0"
                      />
                      <p className="text-xs text-gray-500 mt-1">4-7 transactions, moderate complexity</p>
                    </div>

                    <div>
                      <Label className="flex items-center gap-2">
                        Complex Use Cases
                        <span className="text-xs text-gray-500 font-normal">× 15</span>
                      </Label>
                      <Input
                        type="number"
                        min="0"
                        value={ucpData.complexUseCases}
                        onChange={(e) => {
                          updateScreen(screen.id, {
                            ucpData: {
                              ...ucpData,
                              complexUseCases: Number(e.target.value)
                            }
                          });
                        }}
                        className="mt-1"
                        placeholder="0"
                      />
                      <p className="text-xs text-gray-500 mt-1">&gt;7 transactions, complex logic</p>
                    </div>
                  </div>
                </div>

                {uucp > 0 && (
                  <div className="mt-4 bg-pink-50 border border-pink-200 rounded p-4">
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-3">
                      <div>
                        <span className="text-xs font-semibold text-pink-900 block">Actor Weight</span>
                        <span className="text-lg font-bold text-pink-700">{uaw}</span>
                      </div>
                      <div>
                        <span className="text-xs font-semibold text-pink-900 block">Use Case Weight</span>
                        <span className="text-lg font-bold text-pink-700">{uucw}</span>
                      </div>
                      <div>
                        <span className="text-xs font-semibold text-pink-900 block">Total UUCP</span>
                        <span className="text-lg font-bold text-pink-700">{uucp}</span>
                      </div>
                      <div>
                        <span className="text-xs font-semibold text-pink-900 block">Estimated Time</span>
                        <span className="text-lg font-bold text-pink-700">{estimatedHours}h</span>
                      </div>
                    </div>
                    <p className="text-xs text-pink-600">Using industry standard: 20 hours per Use Case Point</p>
                  </div>
                )}
              </Card>
            );
          })}
        </div>
      )}

      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button onClick={handleNext} className="bg-blue-600 hover:bg-blue-700">
          Continue to Complexity Weighting
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}